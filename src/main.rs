use clap::Parser;

mod types;
mod util;

use types::*;
use util::*;

// to avoid having to relying on reading external file
// // currently contains "name", "decimals", "allowance", and "approve" (this one is not used yet)
static ABI_STR: &'static str = r#"[{"inputs":[],"name":"name","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"decimals","outputs":[{"internalType":"uint8","name":"","type":"uint8"}],"stateMutability":"view","type":"function"},{"name":"allowance","inputs":[{"internalType":"address","name":"owner","type":"address"},{"internalType":"address","name":"spender","type":"address"}],"outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"name":"approve","inputs":[{"internalType":"address","name":"spender","type":"address"},{"internalType":"uint256","name":"amount","type":"uint256"}],"outputs":[{"internalType":"bool","name":"","type":"bool"}],"stateMutability":"nonpayable","type":"function"}]"#;

#[tokio::main]
async fn main() {
    let cmd_args = CommandlineArgs::parse();

    // validate value of chain flag option
    let chain_value = cmd_args.chain;
    let mut chain: Option<ChainType> = None;
    if chain_value == "bsc" {
        chain= Some(ChainType::BSC);
    }
    else if chain_value == "ehtereum" {
        chain = Some(ChainType::Ethereum);
    }
    else if chain_value == "polygon" {
        chain = Some(ChainType::Polygon);
    }
    // non-match case will be handled by clap crate
    
    let chain_unwrapped_value = chain.unwrap();
    let web3 = create_web3(chain_unwrapped_value);

    // validate the input contract address
    let is_eoa_res = perform_check_is_eoa(&web3, &cmd_args.contract_address).await;
    match is_eoa_res {
        Ok(is_eoa) => {
            if is_eoa {
                eprintln!("Error, input contract address is EOA");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("{}", format!("Error validating input contract address; err={}", e));
            std::process::exit(1);
        }
    }

    // create a contract instance
    let contract = match create_contract(&web3, &cmd_args.contract_address, &ABI_STR) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{}", format!("Error creating a contract instance; err={}", e));
            std::process::exit(1);
        }
    };

    // for setter (estimate gas - dry run only)
    if cmd_args.dry_run_estimate_gas {
        // --dry-run-estimate-gas requires presence of --ensure-setter
        if !cmd_args.ensure_setter {
            eprintln!("Error, --dry-run-estimate-gas requires --ensure-setter flag");
            std::process::exit(1);
        }

        // check required parameter
        if cmd_args.estimate_gas_from_addr.is_none() {
            eprintln!("Error, requires --estimate-gas-from-addr to be set");
            std::process::exit(1);
        }

        let estimate_gas_from_addr = cmd_args.estimate_gas_from_addr.unwrap();

        let est_gas_used = web3_query_estimate_gas(&contract, &cmd_args.fn_name, cmd_args.params.as_slice(), &estimate_gas_from_addr).await;
        let f_est_gas_used: f64;
        let estimated_gas_used: U256;
        match est_gas_used {
            Ok(res) => {
                estimated_gas_used = res;

                // convert from base U256 to primitive_types's U256 which has floating point
                // feature
                f_est_gas_used = match primitive_types::U256::from_dec_str(&estimated_gas_used.to_string()) {
                    Ok(res) => res.to_f64_lossy(),
                    Err(e) => {
                        eprintln!("Error converting from base U256 to floating-point ready U256; err={}", e);
                        std::process::exit(1);
                    }
                };
            },
            Err(e) => {
                eprintln!("{}", format!("Error estimating gas by calling a setter method '{}'; err={}", &cmd_args.fn_name, e));
                std::process::exit(1);
            }
        };

        // print the gas price
        // so user can mutiply with the unit of gas used from prior
        match web3.eth().gas_price().await {
            Ok(gas_price) => {
                // convert from base U256 to primitive_types's U256 which has floating point
                // feature
                let f_gas_price = match primitive_types::U256::from_dec_str(&gas_price.to_string()) {
                    Ok(res) => res,
                    Err(e) => {
                        eprintln!("Error converting from base U256 to floating-point ready U256; err={}", e);
                        std::process::exit(1);
                    }
                };

                let gas_price = f_gas_price.to_f64_lossy() / 10_f64.powf(18_f64);
                println!("{:?} {} {}", estimated_gas_used, gas_price, gas_price * f_est_gas_used);
            },
            Err(e) => {
                eprintln!("Error in querying gas price; err={}", e);
                std::process::exit(1);
            }
        }
    }
    // for setter (rpc-eth)
    else if cmd_args.ensure_setter && cmd_args.rpc_eth {
        // nothing for now...
        println!("Not support right now");
    }
    // for setter
    else if cmd_args.ensure_setter {
        let tx_receipt_res = web3_query_set(&contract, &cmd_args.fn_name, &cmd_args.params.as_slice(), cmd_args.block_confirmations).await;
        match tx_receipt_res {
            Ok(tx_receipt) => {
                println!("{:?}", tx_receipt.transaction_hash);
            },
            Err(e) => {
                eprintln!("{}", format!("Error calling setter method '{}'; err={}", &cmd_args.fn_name, e));
                std::process::exit(1);
            }
        }
    }
    // for getter (rpc-eth)
    else if cmd_args.rpc_eth {
        // query balance of the target address
        if &cmd_args.fn_name == "balance" {
            let contract_addr = match get_address_from_str(&cmd_args.contract_address) {
                Ok(addr) => addr,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };

            let bal_res = web3.eth().balance(contract_addr, None).await;
            match bal_res {
                Ok(bal) => {
                    let fready_bal = match primitive_types::U256::from_dec_str(&bal.to_string()) {
                        Ok(res) => res,
                        Err(e) => {
                            eprintln!("Error converting from base U256 to floating-point ready U256; err={}", e);
                            std::process::exit(1);
                        }
                    };

                    println!("{:?} {:?}", bal, fready_bal.to_f64_lossy() / 10_f64.powf(18_f64));
                },
                Err(e) => {
                    eprintln!("Error converting from base U256 to floating-point ready U256; err={}", e);
                    std::process::exit(1);
                }
            }
        }
    }
    // for getter
    else {
        let ret_type_str = match cmd_args.fn_ret_type {
            Some(type_str) => type_str,
            None => {
                eprintln!("Error, require --fn-ret-type for interacting with getter method of smart contract");
                std::process::exit(1);
            }
        };

        // make a call to specified function of the target smart contract
        // FIXME: this should be more concise and shorter code...
        if ret_type_str == "String" {
            let res = web3_query_get::<String>(&contract, &cmd_args.fn_name, cmd_args.params.as_slice()).await;
            match res {
                Ok(res) => println!("{}", res),
                Err(e) => {
                    eprintln!("{}", format!("Error querying of method '{}'; err={}", &cmd_args.fn_name, e));
                    std::process::exit(1);
                }
            }
        }
        else if ret_type_str == "U256" {
            let res = web3_query_get::<U256>(&contract, &cmd_args.fn_name, cmd_args.params.as_slice()).await;
            match res {
                Ok(res) => println!("{:?}", res),
                Err(e) => {
                    eprintln!("{}", format!("Error querying of method '{}'; err={}", &cmd_args.fn_name, e));
                    std::process::exit(1);
                }
            }
        }
    }
    // -- the less of non-match cases handled by clap crate
}
