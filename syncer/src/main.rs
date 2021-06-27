use chainseeker_syncer::*;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: {} COIN", args[0]);
        return;
    }
    let coin = args[1].to_string();
    let config = load_config();
    let mut syncer = Syncer::new(&coin, &config).await;
    syncer.initial_sync().await;
    if syncer.is_stopped().await {
        return;
    }
    let server = syncer.http_server.clone();
    let mut handles = Vec::new();
    handles.push(tokio::spawn(async move {
        syncer.run().await;
    }));
    handles.push(tokio::spawn(async move {
        server.run(&config.http_ip, config.coins[&coin].http_port).await;
    }));
    // Join for the threads.
    for handle in handles.iter_mut() {
        handle.await.expect("Failed to await a tokio JoinHandle.");
    }
}
