#[tokio::main]
async fn main() {
    if let Ok((coin, config)) = chainseeker_server::parse_arguments(&std::env::args().collect::<Vec<String>>()) {
        chainseeker_server::main(&coin, &config).await;
    }
}
