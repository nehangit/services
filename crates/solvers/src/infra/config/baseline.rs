use {
    crate::{
        domain::{eth, solver::baseline},
        infra::{config::unwrap_or_log, contracts},
        util::serialize,
    },
    chain::Chain,
    ethereum_types::H160,
    serde::Deserialize,
    serde_with::serde_as,
    shared::price_estimation::gas::SETTLEMENT_OVERHEAD,
    std::path::Path,
    tokio::fs,
};

#[serde_as]
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct Config {
    /// Optional chain ID. This is used to automatically determine the address
    /// of the WETH contract.
    chain_id: Option<Chain>,

    /// Optional WETH contract address. This can be used to specify a manual
    /// value **instead** of using the canonical WETH contract for the
    /// configured chain.
    weth: Option<H160>,

    /// List of base tokens to use when path finding. This defines the tokens
    /// that can appear as intermediate "hops" within a trading route. Note that
    /// WETH is always considered as a base token.
    base_tokens: Vec<eth::H160>,

    /// The maximum number of hops to consider when finding the optimal trading
    /// path.
    max_hops: usize,

    /// The maximum number of pieces to divide partially fillable limit orders
    /// when trying to solve it against baseline liquidity.
    max_partial_attempts: usize,

    /// Units of gas that get added to the gas estimate for executing a
    /// computed trade route to arrive at a gas estimate for a whole settlement.
    #[serde(default = "default_gas_offset")]
    solution_gas_offset: i64,

    /// The amount of the native token to use to estimate native price of a
    /// token
    #[serde_as(as = "serialize::U256")]
    native_token_price_estimation_amount: eth::U256,
}

/// Load the driver configuration from a TOML file.
///
/// # Panics
///
/// This method panics if the config is invalid or on I/O errors.
pub async fn load(path: &Path) -> baseline::Config {
    let data = fs::read_to_string(path)
        .await
        .unwrap_or_else(|e| panic!("I/O error while reading {path:?}: {e:?}"));
    // Not printing detailed error because it could potentially leak secrets.
    let config = unwrap_or_log(toml::de::from_str::<Config>(&data), &path);
    let weth = match (config.chain_id, config.weth) {
        (Some(chain_id), None) => contracts::Contracts::for_chain(chain_id).weth,
        (None, Some(weth)) => eth::WethAddress(weth),
        (Some(_), Some(_)) => panic!(
            "invalid configuration: cannot specify both `chain-id` and `weth` configuration \
             options",
        ),
        (None, None) => panic!(
            "invalid configuration: must specify either `chain-id` or `weth` configuration options",
        ),
    };

    baseline::Config {
        weth,
        base_tokens: config
            .base_tokens
            .into_iter()
            .map(eth::TokenAddress)
            .collect(),
        max_hops: config.max_hops,
        max_partial_attempts: config.max_partial_attempts,
        solution_gas_offset: config.solution_gas_offset.into(),
        native_token_price_estimation_amount: config.native_token_price_estimation_amount,
    }
}

/// Returns minimum gas used for settling a single order.
/// (not accounting for the cost of additional interactions)
fn default_gas_offset() -> i64 {
    SETTLEMENT_OVERHEAD.try_into().unwrap()
}
