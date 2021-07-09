use std::env::args;
use std::process::exit;
use chainseeker::serde;
use chainseeker::reqwest;

async fn execute<S, F, E>(args: &[String], n_params: usize, exec: E) -> Result<(), String>
    where S: serde::Serialize,
          F: std::future::Future<Output = Result<S, reqwest::Error>>,
          E: Fn() -> F,
{
    if args.len() < n_params + 3 {
        return Err("E: insufficient number of arguments.".to_string());
    }
    match exec().await {
        Ok(result) => println!("{}", serde_json::to_string_pretty(&result).unwrap()),
        Err(err) => return Err(format!("E: failed to execute: {}", err)),
    }
    Ok(())
}

async fn run(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        println!("Usage: {} COIN COMMAND [..ARGS]", args[0]);
        println!("Available COMMANDs are:");
        println!("    status");
        println!("    tx TXID");
        println!("    puttx RAWTX");
        println!("    blocksummary OFFSET LIMIT");
        println!("    block+txids HASH_OR_HEIGHT");
        println!("    block+txs HASH_OR_HEIGHT");
        println!("    block HASH_OR_HEIGHT");
        println!("    txids ADDRESS");
        println!("    txs ADDRESS");
        println!("    utxos ADDRESS");
        println!("    richlistcount");
        println!("    rank ADDRESS");
        println!("    richlist OFFSET LIMIT");
        return Ok(());
    }
    let coin = &args[1];
    let command = &args[2];
    let i = 3;
    let client = chainseeker::new(&format!("https://{}-v3.chainseeker.info/api", coin));
    match command.as_str() {
        "status"        => execute(&args, 0, || client.status()).await,
        "tx"            => execute(&args, 1, || client.tx(&args[i])).await,
        "puttx"         => execute(&args, 1, || client.put_tx(args[i].to_string())).await,
        "blocksummary"  => execute(&args, 2, || client.block_summary(args[i].parse().unwrap(), args[i+1].parse().unwrap())).await,
        "block+txids"   => execute(&args, 1, || client.block_with_txids(&args[i])).await,
        "block+txs"     => execute(&args, 1, || client.block_with_txs(&args[i])).await,
        "block"         => execute(&args, 1, || client.block_header(&args[i])).await,
        "txids"         => execute(&args, 1, || client.txids(&args[i])).await,
        "txs"           => execute(&args, 1, || client.txs(&args[i])).await,
        "utxos"         => execute(&args, 1, || client.utxos(&args[i])).await,
        "richlistcount" => execute(&args, 0, || client.rich_list_count()).await,
        "rank"          => execute(&args, 1, || client.rich_list_addr_rank(&args[i])).await,
        "richlist"      => execute(&args, 2, || client.rich_list(args[i].parse().unwrap(), args[i+1].parse().unwrap())).await,
        _ => Err(format!("E: invalid command: {}", command)),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = run(&args().collect::<Vec<String>>()).await {
        eprintln!("{}", err);
        exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const COIN: &str = "btc";
    const BLOCK_HASH: &str = "00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048";
    const TXID: &str = "0e3e2357e806b6cdb1f70b54c3a3a17b6714ee1f0e68bebb44a74b1efd512098";
    const ADDRESS: &str = "1CounterpartyXXXXXXXXXXXXXXXUWLpVr";
    async fn test(argv: &[&str]) -> Result<(), String> {
        run(&[vec![args().next().unwrap()], argv.iter().map(|s| s.to_string()).collect()].concat()).await
    }
    #[tokio::test]
    async fn no_args() {
        assert!(test(&[]).await.is_ok());
    }
    #[tokio::test]
    async fn invalid_coin() {
        assert!(test(&["invalid", "status"]).await.is_err());
    }
    #[tokio::test]
    async fn invalid_command() {
        assert!(test(&[COIN, "invalid"]).await.is_err());
    }
    #[tokio::test]
    async fn invalid_num_of_args() {
        assert!(test(&[COIN, "tx"]).await.is_err());
    }
    #[tokio::test]
    async fn status() {
        assert!(test(&[COIN, "status"]).await.is_ok());
    }
    #[tokio::test]
    async fn tx() {
        assert!(test(&[COIN, "tx", TXID]).await.is_ok());
    }
    #[tokio::test]
    async fn puttx() {
        // TODO: create a valid transaction.
        assert!(test(&[COIN, "puttx", "012345678"]).await.is_err());
    }
    #[tokio::test]
    async fn blocksummary() {
        assert!(test(&[COIN, "blocksummary", "0", "10"]).await.is_ok());
    }
    #[tokio::test]
    async fn block_txids() {
        assert!(test(&[COIN, "block+txids", BLOCK_HASH]).await.is_ok());
    }
    #[tokio::test]
    async fn block_txs() {
        assert!(test(&[COIN, "block+txs", BLOCK_HASH]).await.is_ok());
    }
    #[tokio::test]
    async fn block() {
        assert!(test(&[COIN, "block", BLOCK_HASH]).await.is_ok());
    }
    #[tokio::test]
    async fn txids() {
        assert!(test(&[COIN, "txids", ADDRESS]).await.is_ok());
    }
    #[tokio::test]
    async fn txs() {
        assert!(test(&[COIN, "txs", ADDRESS]).await.is_ok());
    }
    #[tokio::test]
    async fn utxos() {
        assert!(test(&[COIN, "utxos", ADDRESS]).await.is_ok());
    }
    #[tokio::test]
    async fn richlistcount() {
        assert!(test(&[COIN, "richlistcount"]).await.is_ok());
    }
    #[tokio::test]
    async fn rank() {
        assert!(test(&[COIN, "rank", ADDRESS]).await.is_ok());
    }
    #[tokio::test]
    async fn richlist() {
        assert!(test(&[COIN, "richlist", "0", "10"]).await.is_ok());
    }
}
