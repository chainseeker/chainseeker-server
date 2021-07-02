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
    let coin_config = &config.coins[&coin];
    // Create Syncer instance.
    let mut syncer = Syncer::new(&coin, &coin_config).await;
    let mut handles = Vec::new();
    // Run HTTP server.
    {
        let server = syncer.http_server.clone();
        let http_ip = coin_config.http_ip.clone();
        let http_port = coin_config.http_port;
        handles.push(tokio::spawn(async move {
            server.run(&http_ip, http_port).await;
        }));
    }
    // Run WebSocketRelay.
    {
        let ws = WebSocketRelay::new();
        let zmq_endpoint = coin_config.zmq_endpoint.clone();
        let ws_endpoint = coin_config.ws_endpoint.clone();
        handles.push(tokio::spawn(async move {
            ws.run(&zmq_endpoint, &ws_endpoint).await;
        }));
    }
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
