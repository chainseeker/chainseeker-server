use chainseeker_syncer::*;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("usage: {} COIN", args[0]);
        return;
    }
    let config = load_config();
    let coin = args[1].to_string();
    let mut syncer = Syncer::new(&coin, &config).await;
    let addr_index_db = syncer.addr_index_db();
    let utxo_server = syncer.utxo_server();
    let mut handles = Vec::new();
    handles.push(tokio::spawn(async move {
        syncer.run().await;
    }));
    handles.push(tokio::spawn(async move {
        let server = HttpServer::new(addr_index_db, utxo_server);
        server.run(&coin, &config).await;
    }));
    // Join for the threads.
    for handle in handles.iter_mut() {
        handle.await.expect("Failed to await a tokio JoinHandle.");
    }
}
