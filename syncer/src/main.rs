use chainseeker_syncer::*;

#[tokio::main]
async fn main() {
    // Read arguments.
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: {} COIN", args[0]);
        return;
    }
    // Load config.
    let coin = args[1].to_string();
    let config = load_config();
    // Create Syncer instance.
    let mut syncer = Syncer::new(&coin, &config).await;
    // Run HTTP server.
    let mut handles = Vec::new();
    let server = syncer.http_server.clone();
    handles.push(tokio::spawn(async move {
        server.run(&config.http_ip, config.coins[&coin].http_port).await;
    }));
    // Do initial sync.
    syncer.initial_sync().await;
    if syncer.is_stopped().await {
        return;
    }
    // Run syncer.
    handles.push(tokio::spawn(async move {
        syncer.run().await;
    }));
    // Join for the threads.
    for handle in handles.iter_mut() {
        handle.await.expect("Failed to await a tokio JoinHandle.");
    }
}
