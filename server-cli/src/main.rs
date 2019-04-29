use common::clock::Clock;
use log::info;
use server::{Event, Input, Server};
use std::time::Duration;

const TPS: u64 = 30;

fn main() {
    // Init logging
    pretty_env_logger::init();

    info!("Starting server-cli...");

    // Set up an fps clock
    let mut clock = Clock::new();

    // Create server
    let mut server = Server::new().expect("Failed to create server instance");

    loop {
        let events = server
            .tick(Input::default(), clock.get_last_delta())
            .expect("Failed to tick server");

        for event in events {
            match event {
                Event::ClientConnected { entity } => info!("Client connected!"),
                Event::ClientDisconnected { entity } => info!("Client disconnected!"),
                Event::Chat { entity, msg } => info!("[Client] {}", msg),
            }
        }

        // Clean up the server after a tick
        server.cleanup();

        // Wait for the next tick
        clock.tick(Duration::from_millis(1000 / TPS));
    }
}
