use clap::Parser;
pub use web3::{
    types::{Address, U256},
};

/// Commandline's arguments type
#[derive(Debug, Parser)]
#[clap(author="Wasin Thonkaew (wasin@wasin.io)")]
#[clap(name="crunner")]
#[clap(about="Runner/Executor of target smart contract on EVM-based chain at command line")]
pub struct CommandlineArgs {
    /// Target contract address to interact with
    #[clap(long="address", short='a', required=true, multiple_values=false)]
    pub contract_address: String,

    /// Which chain to work with
    #[clap(long="chain", short='c', required=true, multiple_values=false, possible_values=["bsc", "ethereum", "polygon"], ignore_case=true)]
    pub chain: String,

    /// Function name of target smart contract to make a call to.
    /// To make a query to basic RPC-ETH call, then supply --rpc-eth flag.
    #[clap(long="fn-name", short='f', required=true, multiple_values=false)]
    pub fn_name: String,

    #[clap(long="rpc-eth", multiple_values=false, default_missing_value="true", takes_value=false, conflicts_with_all=&["ensure-setter", "dry-run-estimate-gas"])]
    pub rpc_eth: bool,

    /// Function's returning type
    #[clap(long="fn-ret-type", short='r', multiple_values=false, takes_value=true, possible_values=["String", "U256"], required_unless_present_any=&["ensure-setter", "dry-run-estimate-gas"])]
    pub fn_ret_type: Option<String>,

    /// To ensure that the function to be called is a setter function
    #[clap(long="ensure-setter", multiple_values=false, default_missing_value="true", takes_value=false)]
    pub ensure_setter: bool,

    /// Multiple parameters to be supplied to the function
    #[clap(long="params", short='p', multiple_values=true, takes_value=true)]
    pub params: Vec<String>,

    /// Dry run to estimate gas used for setter method.
    /// It needs `ensure_setter` to be set.
    #[clap(long="dry-run-estimate-gas", multiple_values=false, default_missing_value="true", takes_value=false)]
    pub dry_run_estimate_gas: bool,
    
    /// From address used only for dry-run for estimating gas.
    #[clap(long="estimate-gas-from-addr", multiple_values=false, takes_value=true, required_if_eq("dry-run-estimate-gas", "true"))]
    pub estimate_gas_from_addr: Option<String>,

    #[clap(long="block-confirmations", multiple_values=false, takes_value=true, default_value="20", required_if_eq("ensure-setter", "true"))]
    pub block_confirmations: u64,

    /// ABI filepath to combine with the default one
    #[clap(long="abi-filepath", multiple_values=false, takes_value=true, required_unless_present="rpc-eth")]
    pub abi_filepath: Option<String>,
}

/// Chain type
#[derive(Clone, Copy)]
pub enum ChainType {
    /// BSC - Binance Smart Chain
    BSC,

    /// Ethereum
    Ethereum,

    /// Polygon
    Polygon,
}

/// Type of parameter passed into the method for further processing
///
/// # NOTE
/// This identifies the type for what would be the type for `FnParamWrapperType`
/// which wraps the actual value.
pub enum FnParamType {
    Address,
    String,
    HU256,
    DU256,
}
