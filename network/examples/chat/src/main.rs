use async_std::io;
use clap::{App, Arg};
use futures::executor::{block_on, ThreadPool};
use network::{Address, Network, Participant, Pid, PROMISES_CONSISTENCY, PROMISES_ORDERED};
use std::{sync::Arc, thread, time::Duration};
use tracing::*;
use tracing_subscriber::EnvFilter;
use uvth::ThreadPoolBuilder;

///This example contains a simple chatserver, that allows to send messages
/// between participants
fn main() {
    let matches = App::new("Chat example")
        .version("0.1.0")
        .author("Marcel Märtens <marcel.cochem@googlemail.com>")
        .about("example chat implemented with veloren-network")
        .arg(
            Arg::with_name("mode")
                .short("m")
                .long("mode")
                .takes_value(true)
                .possible_values(&["server", "client", "both"])
                .default_value("both")
                .help(
                    "choose whether you want to start the server or client or both needed for \
                     this program",
                ),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .default_value("52000")
                .help("port to listen on"),
        )
        .arg(
            Arg::with_name("ip")
                .long("ip")
                .takes_value(true)
                .default_value("127.0.0.1")
                .help("ip to listen and connect to"),
        )
        .arg(
            Arg::with_name("protocol")
                .long("protocol")
                .takes_value(true)
                .default_value("tcp")
                .possible_values(&["tcp", "upd", "mpsc"])
                .help(
                    "underlying protocol used for this test, mpsc can only combined with mode=both",
                ),
        )
        .arg(
            Arg::with_name("trace")
                .short("t")
                .long("trace")
                .takes_value(true)
                .default_value("warn")
                .possible_values(&["trace", "debug", "info", "warn", "error"])
                .help("set trace level, not this has a performance impact!"),
        )
        .get_matches();

    let trace = matches.value_of("trace").unwrap();
    let filter = EnvFilter::from_default_env().add_directive(trace.parse().unwrap());
    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(filter)
        .init();

    let port: u16 = matches.value_of("port").unwrap().parse().unwrap();
    let ip: &str = matches.value_of("ip").unwrap();
    let address = match matches.value_of("protocol") {
        Some("tcp") => Address::Tcp(format!("{}:{}", ip, port).parse().unwrap()),
        Some("udp") => Address::Udp(format!("{}:{}", ip, port).parse().unwrap()),
        _ => panic!("invalid mode, run --help!"),
    };

    let mut background = None;
    match matches.value_of("mode") {
        Some("server") => server(address),
        Some("client") => client(address),
        Some("both") => {
            let address1 = address.clone();
            background = Some(thread::spawn(|| server(address1)));
            thread::sleep(Duration::from_millis(200)); //start client after server
            client(address)
        },
        _ => panic!("invalid mode, run --help!"),
    };
    if let Some(background) = background {
        background.join().unwrap();
    }
}

fn server(address: Address) {
    let thread_pool = ThreadPoolBuilder::new().build();
    let server = Arc::new(Network::new(Pid::new(), &thread_pool));
    let pool = ThreadPool::new().unwrap();
    block_on(async {
        server.listen(address).await.unwrap();
        loop {
            let p1 = server.connected().await.unwrap();
            let server1 = server.clone();
            pool.spawn_ok(client_connection(server1, p1));
        }
    });
}

async fn client_connection(network: Arc<Network>, participant: Arc<Participant>) {
    let mut s1 = participant.opened().await.unwrap();
    let username = s1.recv::<String>().await.unwrap();
    println!("[{}] connected", username);
    loop {
        match s1.recv::<String>().await {
            Err(_) => {
                break;
            },
            Ok(msg) => {
                println!("[{}]: {}", username, msg);
                let parts = network.participants().await;
                for p in parts.values() {
                    let mut s = p
                        .open(32, PROMISES_ORDERED | PROMISES_CONSISTENCY)
                        .await
                        .unwrap();
                    s.send((username.clone(), msg.clone())).unwrap();
                }
            },
        }
    }
    println!("[{}] disconnected", username);
}

fn client(address: Address) {
    let thread_pool = ThreadPoolBuilder::new().build();
    let client = Network::new(Pid::new(), &thread_pool);
    let pool = ThreadPool::new().unwrap();

    block_on(async {
        let p1 = client.connect(address.clone()).await.unwrap(); //remote representation of p1
        let mut s1 = p1
            .open(16, PROMISES_ORDERED | PROMISES_CONSISTENCY)
            .await
            .unwrap(); //remote representation of s1
        println!("Enter your username:");
        let mut username = String::new();
        io::stdin().read_line(&mut username).await.unwrap();
        username = username.split_whitespace().collect();
        println!("Your username is: {}", username);
        println!("write /quit to close");
        pool.spawn_ok(read_messages(p1));
        s1.send(username).unwrap();
        loop {
            let mut line = String::new();
            io::stdin().read_line(&mut line).await.unwrap();
            line = line.split_whitespace().collect();
            if line.as_str() == "/quit" {
                println!("goodbye");
                break;
            } else {
                s1.send(line).unwrap();
            }
        }
    });
    thread::sleep(Duration::from_millis(30)); // TODO: still needed for correct shutdown
}

// I am quite lazy, the sending is done in a single stream above, but for
// receiving i open and close a stream per message. this can be done easier but
// this allows me to be quite lazy on the server side and just get a list of
// all participants and send to them...
async fn read_messages(participant: Arc<Participant>) {
    while let Ok(mut s) = participant.opened().await {
        let (username, message) = s.recv::<(String, String)>().await.unwrap();
        println!("[{}]: {}", username, message);
    }
    println!("gracefully shut down");
}
