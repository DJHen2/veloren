use async_std::task;
use task::block_on;
use veloren_network::NetworkError;
mod helper;
use helper::{network_participant_stream, tcp, udp};
use std::io::ErrorKind;
use uvth::ThreadPoolBuilder;
use veloren_network::{Address, Network, Pid, PROMISES_CONSISTENCY, PROMISES_ORDERED};

#[test]
#[ignore]
fn network_20s() {
    let (_, _) = helper::setup(false, 0);
    let (_n_a, _, _, _n_b, _, _) = block_on(network_participant_stream(tcp()));
    std::thread::sleep(std::time::Duration::from_secs(30));
}

#[test]
fn stream_simple() {
    let (_, _) = helper::setup(false, 0);
    let (_n_a, _, mut s1_a, _n_b, _, mut s1_b) = block_on(network_participant_stream(tcp()));

    s1_a.send("Hello World").unwrap();
    assert_eq!(block_on(s1_b.recv()), Ok("Hello World".to_string()));
}

#[test]
fn stream_simple_3msg() {
    let (_, _) = helper::setup(false, 0);
    let (_n_a, _, mut s1_a, _n_b, _, mut s1_b) = block_on(network_participant_stream(tcp()));

    s1_a.send("Hello World").unwrap();
    s1_a.send(1337).unwrap();
    assert_eq!(block_on(s1_b.recv()), Ok("Hello World".to_string()));
    assert_eq!(block_on(s1_b.recv()), Ok(1337));
    s1_a.send("3rdMessage").unwrap();
    assert_eq!(block_on(s1_b.recv()), Ok("3rdMessage".to_string()));
}

#[test]
fn stream_simple_udp() {
    let (_, _) = helper::setup(false, 0);
    let (_n_a, _, mut s1_a, _n_b, _, mut s1_b) = block_on(network_participant_stream(udp()));

    s1_a.send("Hello World").unwrap();
    assert_eq!(block_on(s1_b.recv()), Ok("Hello World".to_string()));
}

#[test]
fn stream_simple_udp_3msg() {
    let (_, _) = helper::setup(false, 0);
    let (_n_a, _, mut s1_a, _n_b, _, mut s1_b) = block_on(network_participant_stream(udp()));

    s1_a.send("Hello World").unwrap();
    s1_a.send(1337).unwrap();
    assert_eq!(block_on(s1_b.recv()), Ok("Hello World".to_string()));
    assert_eq!(block_on(s1_b.recv()), Ok(1337));
    s1_a.send("3rdMessage").unwrap();
    assert_eq!(block_on(s1_b.recv()), Ok("3rdMessage".to_string()));
}

#[test]
#[ignore]
fn tcp_and_udp_2_connections() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (_, _) = helper::setup(true, 0);
    let network = Network::new(Pid::new(), &ThreadPoolBuilder::new().build(), None);
    let remote = Network::new(Pid::new(), &ThreadPoolBuilder::new().build(), None);
    block_on(async {
        remote
            .listen(Address::Tcp("0.0.0.0:2000".parse().unwrap()))
            .await?;
        remote
            .listen(Address::Udp("0.0.0.0:2001".parse().unwrap()))
            .await?;
        let p1 = network
            .connect(Address::Tcp("127.0.0.1:2000".parse().unwrap()))
            .await?;
        let p2 = network
            .connect(Address::Udp("127.0.0.1:2001".parse().unwrap()))
            .await?;
        assert!(std::sync::Arc::ptr_eq(&p1, &p2));
        Ok(())
    })
}

#[test]
fn failed_listen_on_used_ports() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (_, _) = helper::setup(false, 0);
    let network = Network::new(Pid::new(), &ThreadPoolBuilder::new().build(), None);
    let udp1 = udp();
    let tcp1 = tcp();
    block_on(network.listen(udp1.clone()))?;
    block_on(network.listen(tcp1.clone()))?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    let network2 = Network::new(Pid::new(), &ThreadPoolBuilder::new().build(), None);
    let e1 = block_on(network2.listen(udp1));
    let e2 = block_on(network2.listen(tcp1));
    match e1 {
        Err(NetworkError::ListenFailed(e)) if e.kind() == ErrorKind::AddrInUse => (),
        _ => assert!(false),
    };
    match e2 {
        Err(NetworkError::ListenFailed(e)) if e.kind() == ErrorKind::AddrInUse => (),
        _ => assert!(false),
    };
    Ok(())
}

/// There is a bug an impris-desktop-1 which fails the DOC tests,
/// it fails exactly `api_stream_send_main` and `api_stream_recv_main` by
/// deadlocking at different times!
/// So i rather put the same test into a unit test, as my gues is that it's
/// compiler related
#[test]
fn api_stream_send_main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (_, _) = helper::setup(false, 0);
    // Create a Network, listen on Port `2200` and wait for a Stream to be opened,
    // then answer `Hello World`
    let network = Network::new(Pid::new(), &ThreadPoolBuilder::new().build(), None);
    let remote = Network::new(Pid::new(), &ThreadPoolBuilder::new().build(), None);
    block_on(async {
        network
            .listen(Address::Tcp("127.0.0.1:2200".parse().unwrap()))
            .await?;
        let remote_p = remote
            .connect(Address::Tcp("127.0.0.1:2200".parse().unwrap()))
            .await?;
        remote_p
            .open(16, PROMISES_ORDERED | PROMISES_CONSISTENCY)
            .await?;
        let participant_a = network.connected().await?;
        let mut stream_a = participant_a.opened().await?;
        //Send  Message
        stream_a.send("Hello World")?;
        Ok(())
    })
}

#[test]
fn api_stream_recv_main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (_, _) = helper::setup(false, 0);
    // Create a Network, listen on Port `2220` and wait for a Stream to be opened,
    // then listen on it
    let network = Network::new(Pid::new(), &ThreadPoolBuilder::new().build(), None);
    let remote = Network::new(Pid::new(), &ThreadPoolBuilder::new().build(), None);
    block_on(async {
        network
            .listen(Address::Tcp("127.0.0.1:2220".parse().unwrap()))
            .await?;
        let remote_p = remote
            .connect(Address::Tcp("127.0.0.1:2220".parse().unwrap()))
            .await?;
        let mut stream_p = remote_p
            .open(16, PROMISES_ORDERED | PROMISES_CONSISTENCY)
            .await?;
        stream_p.send("Hello World")?;
        let participant_a = network.connected().await?;
        let mut stream_a = participant_a.opened().await?;
        //Send  Message
        println!("{}", stream_a.recv::<String>().await?);
        Ok(())
    })
}
