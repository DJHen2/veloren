use crate::{
    api::{Address, Participant},
    channel::Channel,
    message::OutGoingMessage,
    participant::BParticipant,
    prios::PrioManager,
    protocols::{Protocols, TcpProtocol, UdpProtocol},
    types::{Cid, Frame, Pid, Prio, Sid},
};
use async_std::{
    io, net,
    sync::{Mutex, RwLock},
};
use futures::{
    channel::{mpsc, oneshot},
    executor::ThreadPool,
    future::FutureExt,
    select,
    sink::SinkExt,
    stream::StreamExt,
};
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
use tracing::*;
use tracing_futures::Instrument;
//use futures::prelude::*;

type ParticipantInfo = (
    mpsc::UnboundedSender<(Cid, mpsc::UnboundedSender<Frame>)>,
    mpsc::UnboundedSender<Frame>,
    mpsc::UnboundedSender<(Pid, Sid, Frame)>,
    oneshot::Sender<()>,
);
type UnknownChannelInfo = (
    mpsc::UnboundedSender<Frame>,
    Option<oneshot::Sender<io::Result<Participant>>>,
);

#[derive(Debug)]
struct ControlChannels {
    listen_receiver: mpsc::UnboundedReceiver<(Address, oneshot::Sender<io::Result<()>>)>,
    connect_receiver: mpsc::UnboundedReceiver<(Address, oneshot::Sender<io::Result<Participant>>)>,
    connected_sender: mpsc::UnboundedSender<Participant>,
    shutdown_receiver: oneshot::Receiver<()>,
    prios_sender: std::sync::mpsc::Sender<(Prio, Pid, Sid, OutGoingMessage)>,
}

#[derive(Debug)]
pub struct Scheduler {
    local_pid: Pid,
    closed: AtomicBool,
    pool: Arc<ThreadPool>,
    run_channels: Option<ControlChannels>,
    participants: Arc<RwLock<HashMap<Pid, ParticipantInfo>>>,
    participant_from_channel: Arc<RwLock<HashMap<Cid, Pid>>>,
    channel_ids: Arc<AtomicU64>,
    channel_listener: RwLock<HashMap<Address, oneshot::Sender<()>>>,
    unknown_channels: Arc<RwLock<HashMap<Cid, UnknownChannelInfo>>>,
    prios: Arc<Mutex<PrioManager>>,
}

impl Scheduler {
    pub fn new(
        local_pid: Pid,
    ) -> (
        Self,
        mpsc::UnboundedSender<(Address, oneshot::Sender<io::Result<()>>)>,
        mpsc::UnboundedSender<(Address, oneshot::Sender<io::Result<Participant>>)>,
        mpsc::UnboundedReceiver<Participant>,
        oneshot::Sender<()>,
    ) {
        let (listen_sender, listen_receiver) =
            mpsc::unbounded::<(Address, oneshot::Sender<io::Result<()>>)>();
        let (connect_sender, connect_receiver) =
            mpsc::unbounded::<(Address, oneshot::Sender<io::Result<Participant>>)>();
        let (connected_sender, connected_receiver) = mpsc::unbounded::<Participant>();
        let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();
        let (prios, prios_sender) = PrioManager::new();

        let run_channels = Some(ControlChannels {
            listen_receiver,
            connect_receiver,
            connected_sender,
            shutdown_receiver,
            prios_sender,
        });

        (
            Self {
                local_pid,
                closed: AtomicBool::new(false),
                pool: Arc::new(ThreadPool::new().unwrap()),
                run_channels,
                participants: Arc::new(RwLock::new(HashMap::new())),
                participant_from_channel: Arc::new(RwLock::new(HashMap::new())),
                channel_ids: Arc::new(AtomicU64::new(0)),
                channel_listener: RwLock::new(HashMap::new()),
                unknown_channels: Arc::new(RwLock::new(HashMap::new())),
                prios: Arc::new(Mutex::new(prios)),
            },
            listen_sender,
            connect_sender,
            connected_receiver,
            shutdown_sender,
        )
    }

    pub async fn run(mut self) {
        let (part_out_sender, part_out_receiver) = mpsc::unbounded::<(Cid, Frame)>();
        let (configured_sender, configured_receiver) =
            mpsc::unbounded::<(Cid, Pid, Sid, oneshot::Sender<()>)>();
        let (disconnect_sender, disconnect_receiver) = mpsc::unbounded::<Pid>();
        let (stream_finished_request_sender, stream_finished_request_receiver) = mpsc::unbounded();
        let run_channels = self.run_channels.take().unwrap();

        futures::join!(
            self.listen_manager(
                run_channels.listen_receiver,
                part_out_sender.clone(),
                configured_sender.clone(),
            ),
            self.connect_manager(
                run_channels.connect_receiver,
                part_out_sender,
                configured_sender,
            ),
            self.disconnect_manager(disconnect_receiver,),
            self.send_outgoing(),
            self.stream_finished_manager(stream_finished_request_receiver),
            self.shutdown_manager(run_channels.shutdown_receiver),
            self.handle_frames(part_out_receiver),
            self.channel_configurer(
                run_channels.connected_sender,
                configured_receiver,
                disconnect_sender,
                run_channels.prios_sender.clone(),
                stream_finished_request_sender.clone(),
            ),
        );
    }

    async fn listen_manager(
        &self,
        mut listen_receiver: mpsc::UnboundedReceiver<(Address, oneshot::Sender<io::Result<()>>)>,
        part_out_sender: mpsc::UnboundedSender<(Cid, Frame)>,
        configured_sender: mpsc::UnboundedSender<(Cid, Pid, Sid, oneshot::Sender<()>)>,
    ) {
        trace!("start listen_manager");
        while let Some((address, result_sender)) = listen_receiver.next().await {
            debug!(?address, "got request to open a channel_creator");
            let (end_sender, end_receiver) = oneshot::channel::<()>();
            self.channel_listener
                .write()
                .await
                .insert(address.clone(), end_sender);
            self.pool.spawn_ok(Self::channel_creator(
                self.channel_ids.clone(),
                self.local_pid,
                address.clone(),
                end_receiver,
                self.pool.clone(),
                part_out_sender.clone(),
                configured_sender.clone(),
                self.unknown_channels.clone(),
                result_sender,
            ));
        }
        trace!("stop listen_manager");
    }

    async fn connect_manager(
        &self,
        mut connect_receiver: mpsc::UnboundedReceiver<(
            Address,
            oneshot::Sender<io::Result<Participant>>,
        )>,
        part_out_sender: mpsc::UnboundedSender<(Cid, Frame)>,
        configured_sender: mpsc::UnboundedSender<(Cid, Pid, Sid, oneshot::Sender<()>)>,
    ) {
        trace!("start connect_manager");
        while let Some((addr, pid_sender)) = connect_receiver.next().await {
            match addr {
                Address::Tcp(addr) => {
                    let stream = match net::TcpStream::connect(addr).await {
                        Ok(stream) => stream,
                        Err(e) => {
                            pid_sender.send(Err(e)).unwrap();
                            continue;
                        },
                    };
                    info!("Connecting Tcp to: {}", stream.peer_addr().unwrap());
                    Self::init_protocol(
                        &self.channel_ids,
                        self.local_pid,
                        addr,
                        &self.pool,
                        &part_out_sender,
                        &configured_sender,
                        &self.unknown_channels,
                        Protocols::Tcp(TcpProtocol::new(stream)),
                        Some(pid_sender),
                        false,
                    )
                    .await;
                },
                Address::Udp(addr) => {
                    let socket = match net::UdpSocket::bind("0.0.0.0:0").await {
                        Ok(socket) => Arc::new(socket),
                        Err(e) => {
                            pid_sender.send(Err(e)).unwrap();
                            continue;
                        },
                    };
                    if let Err(e) = socket.connect(addr).await {
                        pid_sender.send(Err(e)).unwrap();
                        continue;
                    };
                    info!("Connecting Udp to: {}", addr);
                    let (udp_data_sender, udp_data_receiver) = mpsc::unbounded::<Vec<u8>>();
                    let protocol =
                        Protocols::Udp(UdpProtocol::new(socket.clone(), addr, udp_data_receiver));
                    self.pool.spawn_ok(
                        Self::udp_single_channel_connect(socket.clone(), udp_data_sender)
                            .instrument(tracing::info_span!("udp", ?addr)),
                    );
                    Self::init_protocol(
                        &self.channel_ids,
                        self.local_pid,
                        addr,
                        &self.pool,
                        &part_out_sender,
                        &configured_sender,
                        &self.unknown_channels,
                        protocol,
                        Some(pid_sender),
                        true,
                    )
                    .await;
                },
                _ => unimplemented!(),
            }
        }
        trace!("stop connect_manager");
    }

    async fn disconnect_manager(&self, mut disconnect_receiver: mpsc::UnboundedReceiver<Pid>) {
        trace!("start disconnect_manager");
        while let Some(pid) = disconnect_receiver.next().await {
            //Closing Participants is done the following way:
            // 1. We drop our senders and receivers
            // 2. we need to close BParticipant, this will drop its senderns and receivers
            // 3. Participant will try to access the BParticipant senders and receivers with
            // their next api action, it will fail and be closed then.
            if let Some((_, _, _, sender)) = self.participants.write().await.remove(&pid) {
                sender.send(()).unwrap();
            }
        }
        trace!("stop disconnect_manager");
    }

    async fn send_outgoing(&self) {
        //This time equals the MINIMUM Latency in average, so keep it down and //Todo:
        // make it configureable or switch to await E.g. Prio 0 = await, prio 50
        // wait for more messages
        const TICK_TIME: std::time::Duration = std::time::Duration::from_millis(10);
        const FRAMES_PER_TICK: usize = 1000000;
        trace!("start send_outgoing");
        while !self.closed.load(Ordering::Relaxed) {
            let mut frames = VecDeque::new();
            self.prios
                .lock()
                .await
                .fill_frames(FRAMES_PER_TICK, &mut frames);
            for (pid, sid, frame) in frames {
                if let Some((_, _, sender, _)) = self.participants.write().await.get_mut(&pid) {
                    sender.send((pid, sid, frame)).await.unwrap();
                }
            }
            async_std::task::sleep(TICK_TIME).await;
        }
        trace!("stop send_outgoing");
    }

    async fn handle_frames(&self, mut part_out_receiver: mpsc::UnboundedReceiver<(Cid, Frame)>) {
        trace!("start handle_frames");
        while let Some((cid, frame)) = part_out_receiver.next().await {
            trace!("handling frame");
            if let Some(pid) = self.participant_from_channel.read().await.get(&cid) {
                if let Some((_, sender, _, _)) = self.participants.write().await.get_mut(&pid) {
                    sender.send(frame).await.unwrap();
                }
            } else {
                error!("dropping frame, unreachable, got a frame from a non existing channel");
            }
        }
        trace!("stop handle_frames");
    }

    //
    async fn channel_configurer(
        &self,
        mut connected_sender: mpsc::UnboundedSender<Participant>,
        mut receiver: mpsc::UnboundedReceiver<(Cid, Pid, Sid, oneshot::Sender<()>)>,
        disconnect_sender: mpsc::UnboundedSender<Pid>,
        prios_sender: std::sync::mpsc::Sender<(Prio, Pid, Sid, OutGoingMessage)>,
        stream_finished_request_sender: mpsc::UnboundedSender<(Pid, Sid, oneshot::Sender<()>)>,
    ) {
        trace!("start channel_activator");
        while let Some((cid, pid, offset_sid, sender)) = receiver.next().await {
            if let Some((frame_sender, pid_oneshot)) =
                self.unknown_channels.write().await.remove(&cid)
            {
                trace!(
                    ?cid,
                    ?pid,
                    "detected that my channel is ready!, activating it :)"
                );
                let mut participants = self.participants.write().await;
                if !participants.contains_key(&pid) {
                    debug!(?cid, "new participant connected via a channel");
                    let (
                        bparticipant,
                        stream_open_sender,
                        stream_opened_receiver,
                        mut transfer_channel_receiver,
                        frame_recv_sender,
                        frame_send_sender,
                        shutdown_sender,
                    ) = BParticipant::new(
                        pid,
                        offset_sid,
                        prios_sender.clone(),
                        stream_finished_request_sender.clone(),
                    );

                    let participant = Participant::new(
                        self.local_pid,
                        pid,
                        stream_open_sender,
                        stream_opened_receiver,
                        disconnect_sender.clone(),
                    );
                    if let Some(pid_oneshot) = pid_oneshot {
                        // someone is waiting with connect, so give them their PID
                        pid_oneshot.send(Ok(participant)).unwrap();
                    } else {
                        // noone is waiting on this Participant, return in to Network
                        connected_sender.send(participant).await.unwrap();
                    }
                    transfer_channel_receiver
                        .send((cid, frame_sender))
                        .await
                        .unwrap();
                    participants.insert(
                        pid,
                        (
                            transfer_channel_receiver,
                            frame_recv_sender,
                            frame_send_sender,
                            shutdown_sender,
                        ),
                    );
                    self.participant_from_channel.write().await.insert(cid, pid);
                    self.pool.spawn_ok(
                        bparticipant
                            .run()
                            .instrument(tracing::info_span!("participant", ?pid)),
                    );
                } else {
                    error!(
                        "2ND channel of participants opens, but we cannot verify that this is not \
                         a attack to "
                    )
                }
                sender.send(()).unwrap();
            }
        }
        trace!("stop channel_activator");
    }

    // requested by participant when stream wants to close from api, checking if no
    // more msg is in prio and return
    pub(crate) async fn stream_finished_manager(
        &self,
        mut stream_finished_request_receiver: mpsc::UnboundedReceiver<(
            Pid,
            Sid,
            oneshot::Sender<()>,
        )>,
    ) {
        trace!("start stream_finished_manager");
        while let Some((pid, sid, sender)) = stream_finished_request_receiver.next().await {
            //TODO: THERE MUST BE A MORE CLEVER METHOD THAN SPIN LOCKING! LIKE REGISTERING
            // DIRECTLY IN PRIO AS A FUTURE WERE PRIO IS WAKER! TODO: also this
            // has a great potential for handing network, if you create a network, send
            // gigabytes close it then. Also i need a Mutex, which really adds
            // to cost if alot strems want to close
            let prios = self.prios.clone();
            self.pool
                .spawn_ok(Self::stream_finished_waiter(pid, sid, sender, prios));
        }
    }

    async fn stream_finished_waiter(
        pid: Pid,
        sid: Sid,
        sender: oneshot::Sender<()>,
        prios: Arc<Mutex<PrioManager>>,
    ) {
        const TICK_TIME: std::time::Duration = std::time::Duration::from_millis(5);
        //TODO: ARRRG, i need to wait for AT LEAST 1 TICK, because i am lazy i just
        // wait 15mn and tick count is 10ms because recv is only done with a
        // tick and not async as soon as we send....
        async_std::task::sleep(TICK_TIME * 3).await;
        let mut n = 0u64;
        loop {
            if !prios.lock().await.contains_pid_sid(pid, sid) {
                trace!("prio is clear, go to close stream as requested from api");
                sender.send(()).unwrap();
                break;
            }
            n += 1;
            if n > 200 {
                warn!(
                    ?pid,
                    ?sid,
                    ?n,
                    "cant close stream, as it still queued, even after 1000ms, this starts to \
                     take long"
                );
                async_std::task::sleep(TICK_TIME * 50).await;
            } else {
                async_std::task::sleep(TICK_TIME).await;
            }
        }
    }

    pub(crate) async fn shutdown_manager(&self, receiver: oneshot::Receiver<()>) {
        trace!("start shutdown_manager");
        receiver.await.unwrap();
        self.closed.store(true, Ordering::Relaxed);
        debug!("shutting down all BParticipants gracefully");
        let mut participants = self.participants.write().await;
        for (pid, (_, _, _, sender)) in participants.drain() {
            trace!(?pid, "shutting down BParticipants");
            sender.send(()).unwrap();
        }
        trace!("stop shutdown_manager");
    }

    pub(crate) async fn channel_creator(
        channel_ids: Arc<AtomicU64>,
        local_pid: Pid,
        addr: Address,
        end_receiver: oneshot::Receiver<()>,
        pool: Arc<ThreadPool>,
        part_out_sender: mpsc::UnboundedSender<(Cid, Frame)>,
        configured_sender: mpsc::UnboundedSender<(Cid, Pid, Sid, oneshot::Sender<()>)>,
        unknown_channels: Arc<RwLock<HashMap<Cid, UnknownChannelInfo>>>,
        result_sender: oneshot::Sender<io::Result<()>>,
    ) {
        info!(?addr, "start up channel creator");
        match addr {
            Address::Tcp(addr) => {
                let listener = match net::TcpListener::bind(addr).await {
                    Ok(listener) => {
                        result_sender.send(Ok(())).unwrap();
                        listener
                    },
                    Err(e) => {
                        info!(
                            ?addr,
                            ?e,
                            "listener couldn't be started due to error on tcp bind"
                        );
                        result_sender.send(Err(e)).unwrap();
                        return;
                    },
                };
                trace!(?addr, "listener bound");
                let mut incoming = listener.incoming();
                let mut end_receiver = end_receiver.fuse();
                while let Some(stream) = select! {
                    next = incoming.next().fuse() => next,
                    _ = end_receiver => None,
                } {
                    let stream = stream.unwrap();
                    info!("Accepting Tcp from: {}", stream.peer_addr().unwrap());
                    Self::init_protocol(
                        &channel_ids,
                        local_pid,
                        addr,
                        &pool,
                        &part_out_sender,
                        &configured_sender,
                        &unknown_channels,
                        Protocols::Tcp(TcpProtocol::new(stream)),
                        None,
                        true,
                    )
                    .await;
                }
            },
            Address::Udp(addr) => {
                let socket = match net::UdpSocket::bind(addr).await {
                    Ok(socket) => {
                        result_sender.send(Ok(())).unwrap();
                        Arc::new(socket)
                    },
                    Err(e) => {
                        info!(
                            ?addr,
                            ?e,
                            "listener couldn't be started due to error on udp bind"
                        );
                        result_sender.send(Err(e)).unwrap();
                        return;
                    },
                };
                trace!(?addr, "listener bound");
                // receiving is done from here and will be piped to protocol as UDP does not
                // have any state
                let mut listeners = HashMap::new();
                let mut end_receiver = end_receiver.fuse();
                let mut data = [0u8; 9216];
                while let Ok((size, remote_addr)) = select! {
                    next = socket.recv_from(&mut data).fuse() => next,
                    _ = end_receiver => Err(std::io::Error::new(std::io::ErrorKind::Other, "")),
                } {
                    let mut datavec = Vec::with_capacity(size);
                    datavec.extend_from_slice(&data[0..size]);
                    if !listeners.contains_key(&remote_addr) {
                        info!("Accepting Udp from: {}", &remote_addr);
                        let (udp_data_sender, udp_data_receiver) = mpsc::unbounded::<Vec<u8>>();
                        listeners.insert(remote_addr.clone(), udp_data_sender);
                        let protocol = Protocols::Udp(UdpProtocol::new(
                            socket.clone(),
                            remote_addr,
                            udp_data_receiver,
                        ));
                        Self::init_protocol(
                            &channel_ids,
                            local_pid,
                            addr,
                            &pool,
                            &part_out_sender,
                            &configured_sender,
                            &unknown_channels,
                            protocol,
                            None,
                            true,
                        )
                        .await;
                    }
                    let udp_data_sender = listeners.get_mut(&remote_addr).unwrap();
                    udp_data_sender.send(datavec).await.unwrap();
                }
            },
            _ => unimplemented!(),
        }
        info!(?addr, "ending channel creator");
    }

    pub(crate) async fn udp_single_channel_connect(
        socket: Arc<net::UdpSocket>,
        mut udp_data_sender: mpsc::UnboundedSender<Vec<u8>>,
    ) {
        let addr = socket.local_addr();
        info!(?addr, "start udp_single_channel_connect");
        //TODO: implement real closing
        let (_end_sender, end_receiver) = oneshot::channel::<()>();

        // receiving is done from here and will be piped to protocol as UDP does not
        // have any state
        let mut end_receiver = end_receiver.fuse();
        let mut data = [0u8; 9216];
        while let Ok(size) = select! {
            next = socket.recv(&mut data).fuse() => next,
            _ = end_receiver => Err(std::io::Error::new(std::io::ErrorKind::Other, "")),
        } {
            let mut datavec = Vec::with_capacity(size);
            datavec.extend_from_slice(&data[0..size]);
            udp_data_sender.send(datavec).await.unwrap();
        }
        info!(?addr, "stop udp_single_channel_connect");
    }

    async fn init_protocol(
        channel_ids: &Arc<AtomicU64>,
        local_pid: Pid,
        addr: std::net::SocketAddr,
        pool: &Arc<ThreadPool>,
        part_out_sender: &mpsc::UnboundedSender<(Cid, Frame)>,
        configured_sender: &mpsc::UnboundedSender<(Cid, Pid, Sid, oneshot::Sender<()>)>,
        unknown_channels: &Arc<RwLock<HashMap<Cid, UnknownChannelInfo>>>,
        protocol: Protocols,
        pid_sender: Option<oneshot::Sender<io::Result<Participant>>>,
        send_handshake: bool,
    ) {
        let (mut part_in_sender, part_in_receiver) = mpsc::unbounded::<Frame>();
        //channels are unknown till PID is known!
        /* When A connects to a NETWORK, we, the listener answers with a Handshake.
          Pro: - Its easier to debug, as someone who opens a port gets a magic number back!
          Contra: - DOS posibility because we answer fist
                  - Speed, because otherwise the message can be send with the creation
        */
        let cid = channel_ids.fetch_add(1, Ordering::Relaxed);
        let channel = Channel::new(cid, local_pid);
        if send_handshake {
            channel.send_handshake(&mut part_in_sender).await;
        }
        pool.spawn_ok(
            channel
                .run(
                    protocol,
                    part_in_receiver,
                    part_out_sender.clone(),
                    configured_sender.clone(),
                )
                .instrument(tracing::info_span!("channel", ?addr)),
        );
        unknown_channels
            .write()
            .await
            .insert(cid, (part_in_sender, pid_sender));
    }
}
