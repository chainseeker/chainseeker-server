pub mod address_index;
pub mod utxo;

const REST_ENDPOINT_ENV_NAME: &str = "BITCOIN_REST_ENDPOINT";
const DEFAULT_DATA_DIR: &str = ".chainseeker";

pub fn get_rest() -> bitcoin_rest::Context {
    let endpoint = std::env::var(REST_ENDPOINT_ENV_NAME).unwrap_or(bitcoin_rest::DEFAULT_ENDPOINT.to_string());
    bitcoin_rest::new(&endpoint)
}

pub fn get_data_dir_path() -> Result<String, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    Ok(home + "/" + DEFAULT_DATA_DIR)
}
