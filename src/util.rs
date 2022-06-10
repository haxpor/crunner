use crate::types::{FnParamType, ChainType};
use ethabi::token::Token;
use std::str::FromStr;

use web3::{
    Web3,
    types::{Address, U256, TransactionReceipt},
    transports::http::Http,
    contract::{Contract, Options, tokens::{Detokenize, Tokenizable}},
};
use regex::Regex;

/// RPC endpoint of BSC chain
pub(crate) static BSC_RPC_ENDPOINT: &str = "https://bsc-dataseed.binance.org/";
/// RPC endpoint of Ethereum chain
pub(crate) static ETHEREUM_RPC_ENDPOINT: &str = "https://rpc.ankr.com/eth";
/// RPC endpoint of Polygon chain
pub(crate) static POLYGON_RPC_ENDPOINT: &str = "https://polygon-rpc.com/";

/// Parse the input param string into type
pub fn parse_param_type(param_str: &str) -> FnParamType {
    // check if it's Address type
    if validate_address_format(param_str) {
        return FnParamType::Address;
    }
    // check if it's hexadecimal type
    else if validate_hexadecimal_format(param_str) {
        return FnParamType::HU256;
    }
    // check if it's decimal type
    else if validate_decimal_format(param_str) {
        return FnParamType::DU256;
    }
    // else it would be string
    else {
        return FnParamType::String;
    }
}

/// Validate whether the specified address is in correct format.
/// Return true if the format is correct, otherwise return false.
///
/// # Arguments
/// * `address` - address to check its format correctness
pub fn validate_address_format(address: &str) -> bool {
    let lowercase_address = address.to_lowercase();
    let regex: Regex = Regex::new(r#"^(0x)?[0-9a-f]{40}$"#).unwrap();

    regex.is_match(&lowercase_address)
}

/// Validate whether the specified string is in hexadecimal format.
///
/// # Arguments
/// - `s` - numeric string to parse
pub fn validate_hexadecimal_format(s: &str) -> bool {
    let lowercase_s = s.to_lowercase();
    let regex: Regex = Regex::new(r#"0x[0-9a-fA-F]+"#).unwrap();

    return regex.is_match(&lowercase_s);
}

/// Validate whether the specified string is in octal format.
///
/// # Arguments
/// - `s` - numeric string to parse
pub fn validate_octal_format(s: &str) -> bool {
    let lowercase_s = s.to_lowercase();
    let regex: Regex = Regex::new(r#"0o[0-7]+"#).unwrap();

    return regex.is_match(&lowercase_s);
}

/// Validate whether the specified string is in decimal format.
///
/// # Arguments
/// - `s` - numeric string to parse
pub fn validate_decimal_format(s: &str) -> bool {
    let lowercase_s = s.to_lowercase();
    let regex: Regex = Regex::new(r#"[-]?[1-9][0-9]*"#).unwrap();

    return regex.is_match(&lowercase_s);
}

/// Perform check whether the specified address is an EOA.
/// Return true if it is, otherwise return false.
///
/// # Arguments
/// * `web3` - instance of web3
/// * `address` - address to check; in format `0x...`.
pub async fn perform_check_is_eoa(web3: &Web3<Http>, address: &str) -> Result<bool, String> {
    if !validate_address_format(address) {
        return Err(format!("Error address is not in the correct format; addr={}", address));
    }

    // convert into hex bytes in order to create `web3::Address`
    let address_hexbytes_decoded = match hex::decode(&address[2..]) {
        Ok(res) => res,
        Err(e) => {
            let err_msg = format!("Error hex decoding of address ({}); err={}", address, e);
            return Err(err_msg);
        }
    };
    
    // query for code
    let code_bytes = match web3.eth().code(Address::from_slice(address_hexbytes_decoded.as_slice()), None).await {
        Ok(res) => res,
        Err(e) => {
            let err_msg = format!("Error awaiting result for code from address ({}); err={}", address, e);
            return Err(err_msg);
        }
    };

    // encode hex bytes into hex string
    let code_str = hex::encode(code_bytes.0.as_slice());

    if code_str.len() > 0 {
        // it is a contract address
        return Ok(false);
    }

    Ok(true)
}

/// Get `Address` from string literal.
///
/// # Arguments
/// * `address` - address string literal prefixed with '0x'
pub fn get_address_from_str(address: &str) -> Result<Address, String> {
    if !validate_address_format(address) {
        return Err(format!("Error address is not in the correct format; addr={}", address));
    }
    
    Ok(Address::from_slice(hex::decode(&address[2..]).unwrap().as_slice()))
}

/// Create a web3 instance
///
/// # Arguments
/// - `chain` - `ChainType`
pub fn create_web3(chain: ChainType) -> Web3<Http> {
    let rpc_endpoint = match chain {
        ChainType::BSC => BSC_RPC_ENDPOINT,
        ChainType::Ethereum => ETHEREUM_RPC_ENDPOINT,
        ChainType::Polygon => POLYGON_RPC_ENDPOINT,
    };
    let http = Http::new(rpc_endpoint).unwrap();
    Web3::new(http)
}

/// Get unit string from the specified `ChainType`.
///
/// # Arguments
/// - `chain` - `ChainType`
///
/// # Return
/// Return static string representing of the chain.
pub fn unit_str(chain: ChainType) -> &'static str {
    match chain {
        ChainType::BSC => "BNB",
        ChainType::Ethereum => "ETH",
        ChainType::Polygon => "MATIC",
    }
}

/// Parse a long hex string into vector of hex string of 64 characters in length (256 bit)
/// excluding the prefixed method-id which has 8 characters in length (32 bit).
/// Return a vector of hex string of 64 characters in length (256 bit);
///
/// # Arguments
/// * `long_hex_str` - input long hex string to parse; included a prefix of `0x`
pub fn parse_256_method_arguments(long_hex_str: &str) -> Result<Vec<String>, String> {
    if long_hex_str.len() == 0 {
        return Ok(Vec::new());
    }

    // get slice excluding prefix of method-id
    let arguments_hex_str = &long_hex_str[10..];

    // the length of input stringis not long enough to get at least one element
    if arguments_hex_str.len() < 64 {
        return Err("Input hex string length is not long enough to be parsed.
It needs to have at least 64 characters in length included with prefix of 0x".to_owned());
    }

    let mut offset_i: usize = 0;
    let mut res_vec: Vec<String> = Vec::new();

    while offset_i + 64 <= arguments_hex_str.len() {
        res_vec.push((&arguments_hex_str[offset_i..offset_i+64]).to_owned());
        offset_i = offset_i + 64;
    }

    Ok(res_vec)
}

/// Create a contract
///
/// # Arguments
/// * `web3` - web3 instance
/// * `contract_address_str` - contract address string
/// * `abi_str` - abi
pub fn create_contract(web3: &Web3<Http>, contract_address_str: &str, abi_str: &str) -> Result<Contract<Http>, String> {
    if !validate_address_format(contract_address_str) {
        let err_msg = format!("Error address is in wrong format ({}).", contract_address_str);
        return Err(err_msg);
    }
    let contract_address_hbytes = match hex::decode(&contract_address_str[2..]) {
        Ok(res) => res,
        Err(e) => return Err(format!("Error converting from literal string of contract address into hex bytes; err={}", e)),
    };
    let contract_address: Address = Address::from_slice(contract_address_hbytes.as_slice());

    // create a contract from contract address, and abi
    match Contract::from_json(web3.eth(), contract_address, abi_str.as_bytes()) {
        Ok(res) => Ok(res),
        Err(e) => {
            let err_msg = format!("Error creating contract associated with abi for {}; err={}", contract_address_str, e);
            Err(err_msg)
        }
    }
}

/// Prepare parameters for supplying to smart contract's method.
///
/// # Arguments
/// - `params` - input parameter strings as slice
/// - `print_param_type` - whether or not to also print each parameter type
///
/// # Return
/// Return a slice of parsed `Token` in case of success.
fn prepare_params(params: &[String], print_param_type: bool) -> Result<Vec<Token>, String> {
    let mut parsed_params: Vec<Token> = Vec::new();

    for p in params {
        if print_param_type {
            print!("param = {}", p);
        }

        match parse_param_type(&p) {
            FnParamType::Address => {
                if print_param_type {
                    println!(" is Address");
                }
                
                let addr = match get_address_from_str(&p) {
                    Ok(addr) => addr,
                    Err(e) => {
                        let err_msg = format!("Error parsing parameter '{}' for Address type; err={}", &p, e);
                        return Err(err_msg);
                    }
                };
                parsed_params.push(addr.into_token());
            },
            FnParamType::HU256 => {
                if print_param_type {
                    println!(" is U256");
                }

                let trimmed_prefix = p.trim_start_matches("0x");
                let u256_val = match U256::from_str_radix(&trimmed_prefix, 16) {
                    Ok(res) => res,
                    Err(e) => {
                        let err_msg = format!("Error creating U256 from hexadecimal string; e={}", e);
                        return Err(err_msg);
                    }
                };
                parsed_params.push(u256_val.into_token());
            }
            FnParamType::DU256 => {
                if print_param_type {
                    println!(" is Decimal");
                }

                let u256_val = match U256::from_dec_str(&p) {
                    Ok(res) => res,
                    Err(e) => {
                        let err_msg = format!("Error creating U256 from decimal string; e={}", e);
                        return Err(err_msg);
                    }
                };
                parsed_params.push(u256_val.into_token());
            },
            FnParamType::String => {
                if print_param_type {
                    println!(" is String");
                }
                parsed_params.push(p.to_owned().into_token());
            }
        }
    }

    Ok(parsed_params)
}

/// Make a web3 query depending on the method name, and number of method's arguments.
///
/// # Arguments
/// - `contract` - `web3::contract::Contract` for contract instance to interact with
/// - `fn_name` - name of the function to make a call
/// - `params` - slice of parameter strings that required to pass to such method to make a call
pub async fn web3_query_get<R>(contract: &Contract<Http>, fn_name: &str, params: &[String]) -> Result<R, String>
where
    R: Detokenize
{
    let parsed_params = match prepare_params(params, false) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let res = contract.query(fn_name, parsed_params.as_slice(), None, Options::default(), None).await;

    match res {
        Ok(val_res) => Ok(val_res),
        Err(e) => Err(format!("Error querying via RPC for function '{}'; err={}", fn_name, e)),
    }
}

/// Make a web3 set depending on the function name, and number of function's arguments.
///
/// # Arguments
/// - `contract` - `web3::contract::Contract` for contract instance to interact with
/// - `fn_name` - name of the function to make a call
/// - `params` - slice of parameter strings that required to pass to such method to make a call
/// - `confirmations` - number of confirmations or number of blocks to be confirmed to report
/// effectively made)
///
/// # Return
/// On success, return `TransactionReceipt`.
pub async fn web3_query_set(contract: &Contract<Http>, fn_name: &str, params: &[String], confirmations: u64) -> Result<TransactionReceipt, String>
{
    let parsed_params = match prepare_params(params, false) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let prvk = secp256k1::SecretKey::from_str(&std::env::var("CRUNNER_SETTER_SECRETKEY").expect("'CRUNNER_SETTER_SECRETKEY' environment variable is required")).unwrap();
    match contract.signed_call_with_confirmations(fn_name, parsed_params.as_slice(), Options::default(), confirmations.try_into().unwrap(), &prvk).await {
        Ok(tx_receipt) => Ok(tx_receipt),
        Err(e) => {
            let err_msg = format!("Error calling setter method namely '{}'; err={}", fn_name, e);
            return Err(err_msg);
        },
    }
}

/// Make a web3 (dry-run for estimate gas) set depending on the function name, and number of function's arguments.
///
/// # Arguments
/// - `contract` - `web3::contract::Contract` for contract instance to interact with
/// - `fn_name` - name of the function to make a call
/// - `params` - slice of parameter strings that required to pass to such method to make a call
/// - `from` - address from
///
/// # Return
/// On success, return `U256` indicating gas used.
pub async fn web3_query_estimate_gas(contract: &Contract<Http>, fn_name: &str, params: &[String], from: &str) -> Result<U256, String>
{
    let parsed_params = match prepare_params(params, false) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let from_addr = match get_address_from_str(from) {
        Ok(addr) => addr,
        Err(e) => return Err(e),
    };

    match contract.estimate_gas(fn_name, parsed_params.as_slice(), from_addr, Options::default()).await {
        Ok(estimated_gas_used) => Ok(estimated_gas_used),
        Err(e) => {
            let err_msg = format!("Error calling setter method namely '{}'; err={}", fn_name, e);
            return Err(err_msg);
        },
    }
}

/// Start measuring time. Suitable for wall-clock time measurement.
/// This is mainly used to measure time of placing a limit order onto Bybit.
pub fn measure_start(start: &mut std::time::Instant) {
    *start = std::time::Instant::now();
}

/// Mark the end of the measurement of time performance.
/// Return result in seconds, along with printing the elapsed time if `also_print`
/// is `true`.
pub fn measure_end(start: &std::time::Instant, also_print: bool) -> f64 {
    let elapsed = start.elapsed().as_secs_f64();
    if also_print {
        println!("(elapsed = {:.2} secs)", elapsed);
    }
    elapsed
}
