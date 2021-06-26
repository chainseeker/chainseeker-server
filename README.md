chainseeker-server
==================

[![Rust](https://github.com/chainseeker/chainseeker-server/actions/workflows/rust.yml/badge.svg)](https://github.com/chainseeker/chainseeker-server/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/chainseeker/chainseeker-server/branch/master/graph/badge.svg?token=MGtM2XKGaD)](https://codecov.io/gh/chainseeker/chainseeker-server)

**chainseeker.info**: fast and reliable open-source cryptocurrency block explorer.

This is a server-side implementation of chainseeker.info.
If you are looking for a client-side library, check [here](https://github.com/chainseeker/chainseeker-client).

Cloning the repo
----------------

```bash
$ git clone https://github.com/chainseeker/chainseeker-syncer.git
```

Prerequisite
------------

**chainseeker-server** requires a Bitcoin Core (or any compatible altcoins) running, and REST and RPC API are enabled.

Configure `bitcoin.conf` as followng:
```toml:bitcoin.conf
rpcuser = YOUR_USERNAME
rpcpassword = VERY_LONG_PASSWORD

server  = 1
txindex = 1
rest    = 1

[main]
zmqpubhashblock = tcp://127.0.0.1:28332
zmqpubhashtx    = tcp://127.0.0.1:28332
zmqpubrawblock  = tcp://127.0.0.1:28332
zmqpubrawtx     = tcp://127.0.0.1:28332

[test]
zmqpubhashblock = tcp://127.0.0.1:28333
zmqpubhashtx    = tcp://127.0.0.1:28333
zmqpubrawblock  = tcp://127.0.0.1:28333
zmqpubrawtx     = tcp://127.0.0.1:28333

[regtest]
zmqpubhashblock = tcp://127.0.0.1:28334
zmqpubhashtx    = tcp://127.0.0.1:28334
zmqpubrawblock  = tcp://127.0.0.1:28334
zmqpubrawtx     = tcp://127.0.0.1:28334

[signet]
zmqpubhashblock = tcp://127.0.0.1:28335
zmqpubhashtx    = tcp://127.0.0.1:28335
zmqpubrawblock  = tcp://127.0.0.1:28335
zmqpubrawtx     = tcp://127.0.0.1:28335
```

Then, launch `bitcoind` or `bitcoin-qt` in your machine.

Configuration
-------------

Copy [./config.example.toml](./config.example.toml) and [./config.example.ts](./config.example.ts)
under the `$HOME/.chainseeker/` directory and edit them.
```bash
$ mkdir $HOME/.chainseeker
$ cp ./config.example.toml $HOME/.chainseeker/config.toml
$ cp ./config.example.ts $HOME/.chainseeker/config.ts
```

Run
---

**chainseeker-server** is consisted of three parts: `syncer`, `api` and `web`.
Each processes should be run in a separate terminal.

### Running syncer

```bash
$ cargo run --release COIN
```

### Running api

```bash
$ npm run api:start
```

### Running web

```bash
$ npm run web:start
```

Setup proxy
-----------

Configure your web server (such as Nginx or Apache) to proxy HTTP requests to the correct port.

1. Proxy `/` to the port of `web` server listening (default port is 7001).
1. Proxy `/api` to the port of `api` server listening (default port is 7002).

