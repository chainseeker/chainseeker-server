#[tokio::main]
async fn main() {
    if let Ok((coin, config)) = chainseeker_server::parse_arguments() {
        chainseeker_server::main(&coin, &config).await;
    }
}
