# Xcavate Loan Contract

## Getting Started

To interact with ink smart contract, please make sure that you have installed the required packages.
Check the [Install](https://docs.astar.network/docs/build/environment/ink_environment/) instructions.

### Build

### Use below command to go the loan directory

```shell
cd contracts/loan
```

### Use below command to build the loan.contract file

```shell
cargo contract build --release
```

### Deploy the contract to the Xcavate node to interact with it

### Run Tests

Run the following command in the loan directory for the rust unit tests in the loan contract.

Comment out the "* 1000000000000" in line 89 and 166 in the lib.rs file of the smart contract for running the tests. These were implmented for demo purpose on polkadot js app. 

```sh
cargo +nightly test
```
