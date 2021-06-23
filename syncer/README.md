chainseeker-syncer
==================

Traverse blockchain data to construct the address index database and UTXO database.

Install
-------

```bash
$ git clone https://github.com/chainseeker/chainseeker-syncer.git
$ cd chainseeker-syncer
$ cargo run
```

Configuration
-------------

This package uses Bitcoin Core's REST APIs.
Enable `rest` feature by adding the following line to your `bitcoin.conf`.

```
txindex = 1
rest = 1
```

Run
---

```bash
$ cargo run
```

...or for faster sync, run with `--release` option.

```bash
$ cargo run --release
```

