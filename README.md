chainseeker-server
==================

[![Rust](https://github.com/chainseeker/chainseeker-server/actions/workflows/rust.yml/badge.svg)](https://github.com/chainseeker/chainseeker-server/actions/workflows/rust.yml)
[![Node.js CI](https://github.com/chainseeker/chainseeker-server/actions/workflows/node.js.yml/badge.svg)](https://github.com/chainseeker/chainseeker-server/actions/workflows/node.js.yml)
[![codecov](https://codecov.io/gh/chainseeker/chainseeker-server/branch/master/graph/badge.svg?token=MGtM2XKGaD)](https://codecov.io/gh/chainseeker/chainseeker-server)

**chainseeker.info**: fast and reliable open-source cryptocurrency block explorer.

This is a server-side implementation of chainseeker.info.
If you are looking for a client-side library, check [here](https://github.com/chainseeker/chainseeker-client).

Cloning the repo
----------------

```bash
$ git clone https://github.com/chainseeker/chainseeker-server.git
```

Prerequisite
------------

**chainseeker-server** requires a Bitcoin Core (or any compatible altcoins) running,
and both REST (for syncing) and RPC API (for broadcasting transactions) are enabled.

Configure `bitcoin.conf` as below:
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

Copy [./config.example.toml](./config.example.toml) under the `$HOME/.chainseeker/` directory and edit them.
```bash
$ mkdir $HOME/.chainseeker
$ cp ./config.example.toml $HOME/.chainseeker/config.toml
```

Install
-------

If you have no Rust environment on your server, install it by running the following command.

```bash
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Run
---

```bash
$ cargo run --release COIN
```

or

```bash
$ npm start COIN
```

The server will traverse all blocks and may take a day to finish syncing.

Setup proxy
-----------

Configure your web server (such as Nginx or Apache) to proxy HTTP requests to the correct port.

1. Proxy `/` to the `http_port` (default port for mainnet Bitcoin is 6000).
1. Proxy `/ws` to the port of `ws_endpoint` (default port is 7000).

And then, configure SSL/TLS certificate if required or use proxy services like [CloudFlare](https://www.cloudflare.com/).

