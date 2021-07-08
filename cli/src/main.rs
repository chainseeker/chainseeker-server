use std::process::exit;
use chainseeker::serde;
use chainseeker::reqwest;

async fn execute<S, F, E>(args: &[String], n_params: usize, exec: E)
    where S: serde::Serialize,
          F: std::future::Future<Output = Result<S, reqwest::Error>>,
          E: Fn() -> F,
{
    if args.len() < n_params + 3 {
        eprintln!("E: insufficient number of arguments.");
        exit(3);
    }
    match exec().await {
        Ok(result) => {
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        },
        Err(err) => {
            eprintln!("E: failed to execute: {}", err);
            exit(2);
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
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
        return;
    }
    let coin = &args[1];
    let command = &args[2];
    let i = 3;
    let client = chainseeker::new(&format!("https://{}-v3.chainseeker.info/api", coin));
    match command.as_str() {
        "status"        => execute(&args, 0, || client.status()).await,
        "tx"            => execute(&args, 1, || client.tx(&args[i])).await,
        "puttx"         => execute(&args, 1, || client.put_tx(args[i].clone())).await,
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
        _ => {
            eprintln!("E: invalid command: {}", command);
            exit(1);
        }
    };
}
