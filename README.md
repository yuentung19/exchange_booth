# exchange_booth

### Before you begin
* Instruction 4, CloseExchangeBooth, is currently unimplemented. Deposit is implemented in the python client.
* only token names 'a' and 'b' are supported
* We tested our implementation on devnet.
* We didn't do anything fancy like accounting for integer overflow, so expect this implementation to not work for all values. We tested with integers < 10.
* Remember you can airdrop yourself some solana if you run out with ```solana airdrop 2``` (use either 2 or 1).

### Build and deploy the application
```
$ cargo build-bpf
$ solana program deploy /Users/tlesniak/solana-bootcamp/exchange_booth/program/target/deploy/exchangebooth.so
```

Note: you should replace the exchangebooth.so path with the corresponding path to your .so file. This should have been output
the build-bpf command.

Once this is deployed, go ahead and cd to the python directory. Then you can run exchange_booth.py, following the interactive prompts. Start with init, then follow with deposit into withdraw or set_rate into exchange.

```
$ python exchange_booth.py program_id
```
