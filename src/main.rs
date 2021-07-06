use chainseeker_server::*;

#[tokio::main]
async fn main() {
    // Read arguments.
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: {} COIN", args[0]);
        return;
    }
    // Load config.
    let coin = &args[1];
    let config = load_config(coin);
    // Create Syncer instance.
    let mut syncer = Syncer::new(&coin, &config).await;
    let mut handles = Vec::new();
    // Run HTTP server.
    {
        let server = syncer.http_server.clone();
        let http_ip = config.http_ip.clone();
        let http_port = config.http_port;
        handles.push(tokio::spawn(async move {
            server.run(&http_ip, http_port).await;
        }));
    }
    // Run WebSocketRelay.
    {
        let ws = WebSocketRelay::new();
        let zmq_endpoint = config.zmq_endpoint.clone();
        let ws_endpoint = config.ws_endpoint.clone();
        handles.push(tokio::spawn(async move {
            ws.run(&zmq_endpoint, &ws_endpoint).await;
        }));
    }
    // Do initial sync.
    syncer.initial_sync().await;
    // Run syncer.
    syncer.run().await;
    // Join for the threads.
    for handle in handles.iter_mut() {
        handle.await.expect("Failed to await a tokio JoinHandle.");
    }
}
