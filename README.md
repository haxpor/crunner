# crunner
Runner/Executor of target smart contract on EVM-based chain at command line.

With `crunner`, you will be able to use command line to make a call to smart contract's
method whether or not such smart contract code is verified or not; provided that
you know its ABI (know function signatures).

# Core features

* Make call to getter/setter methods
* Make call to RPC-ETH query of `balance`
* Make call to get estimated gas for setter methods of the same parameters supplied (dry-run)

# Install

```bash
cargo install crunner
```

# Examples

The following examples are the real commands that you can copy and test it out.

**Be careful** and mindful when execute `Setter call` which is destructive and would
cost gas fee, so better to adapt the example of command to execute your own
transaction.

## Estimate gas

```bash
$ crunner -a 0xa0feB3c81A36E885B6608DF7f0ff69dB97491b58 \
-c bsc \
--fn-name approve \
--ensure-setter \
--params 0x10ed43c718714eb63d5aa57b78b54704e256024e 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff \
--dry-run-estimate-gas \
--estimate-gas-from-addr 0x5a223cf64f95214032d239ed49a6c91eb43d676c
25242 0.000000005 0.00012621
```

This is grabbed from on-chain data as seen from [this transaction](https://bscscan.com/tx/0x3f46e944f81c78ba8bac8c32ce28820df09d919b6cc000eb54525487bf934225).
We just want to estimate gas used for such transaction which results in the similar
amount of gas used, and total gas fees.

The same parameters supplied which would be used when actually execute in real
transaction. It returns 3 numbers separated by a single space respectively as

1. Amount of gas unit used
2. Gas price in unit of native token of such chain in execution i.e. BNB, ETH, or MATIC.
3. Total gas fees (which is = 1. x 2.)

## Getter call

```bash
$ crunner -a 0xbA2aE424d960c26247Dd6c32edC70B295c744C43 \
-c bsc \
--fn-name name \
--fn-ret-type String \
Dogecoin
```

This make a query against a smart contract to get the name of the Dogecoin token
contract. For getter call, it requires `--fn-ret-type` to be supplied which is
`String` in this case.

## Setter call

Take an example of `Estimate gas` with the same of everything except that
we exclude `--dry-run-estimate-gas`, and `--estimate-gas-from-addr` then such
call will be executed on-chain.

The result will shown transaction hash so you can copy it and query it on
indexer website like bscscan.com, etherscan.io, or polygonscan.com.

## RPC-ETH call

```bash
$ crunner -a 0xE2D26507981A4dAaaA8040bae1846C14E0Fb56bF \
-c bsc \
--fn-name balance \
--fn-ret-type U256 \
--rpc-eth
4876566977257765806422 4876.566977257766
```

Make a RPC-ETH query for balance of the target contract address (in this case, it
is BakedBeans contract). Note you need `--rpc-eth` flag in order to toggle `--fn-name`
to be based on RPC-ETH method name instead of smart contract's method name.

`--fn-ret-type` is required, and it needs to be `U256` in this case.

Right now, RPC-ETH supports only `balance` query.

Result is shown respectively of balance in Wei, and native token i.e. BNB, ETH, or MATIC.

# License
MIT, Wasin Thonkaew
