<img src="https://raw.githubusercontent.com/chainseeker/chainseeker-server/master/assets/img/logo-wide.png" alt="chainseeker-server" width="100%" />

[![Rust](https://github.com/chainseeker/chainseeker-server/actions/workflows/rust.yml/badge.svg)](https://github.com/chainseeker/chainseeker-server/actions/workflows/rust.yml)
[![Node.js CI](https://github.com/chainseeker/chainseeker-server/actions/workflows/node.js.yml/badge.svg)](https://github.com/chainseeker/chainseeker-server/actions/workflows/node.js.yml)
[![codecov](https://codecov.io/gh/chainseeker/chainseeker-server/branch/master/graph/badge.svg?token=MGtM2XKGaD)](https://codecov.io/gh/chainseeker/chainseeker-server)

**chainseeker.info**: fast and reliable open-source cryptocurrency block explorer.

This is the server-side implementation of chainseeker.info.
If you are looking for a client-side library, check [here](https://github.com/chainseeker/chainseeker-client).

Features
--------

- 100% open source.
	- Licensed with MIT.
	- No fee, no backdoors and no access limitations (if you serve your server your own).
- Server-side code is fully written in Rust and thus fast.
	- For benchmark results, see [Performance](#performance) section.
	- Can sync to the Bitcoin mainnet in a day (may vary with the machine spec).
- Web front-end is written in Nuxt.js (Vue.js) and statically generated.
	- Can serve with AWS S3 or any other object storage services. No need for web servers.
	- Modern UI interface with [Vuetify](https://vuetifyjs.com/).
- Can serve Bitcoin mainnet with machines with less than 16GB of memory.
	- [chainseeker.info](https://chainseeker.info/) is served on the DigitalOcean's 16GB Memory-Optimized instance.
	- Start from $160 / mo ($80 / mo for instance and $80 / mo for block storage).
- Neet REST API (JSON) interface which can easily interact with your app.
	- JavaScript (TypeScript) client is available [here](https://github.com/chainseeker/chainseeker-client).
	- See [types.ts](https://github.com/chainseeker/chainseeker-client/blob/master/src/types.ts) for JSON interface.
- Other features:
	- Serve the list of transactions related to a given address (so-called "address index").
	- Serve the ranking of addresses (so-called "rich list").
	- Altcoin support (tested with Monacoin).

Cloning the repo
----------------

```bash
$ git clone https://github.com/chainseeker/chainseeker-server.git
```

Prerequisite
------------

**chainseeker-server** requires a Bitcoin Core (or any compatible altcoins) running,
and both REST (for syncing) and RPC API (for broadcasting transactions) being enabled.

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

1. Proxy `/` to the `http_port` (default: 8000).
1. Proxy `/ws` to the port of `ws_endpoint` (default: 8001).

And then, configure SSL/TLS certificate if required or use proxy services like [CloudFlare](https://www.cloudflare.com/).

Performance
-----------

### Server specs

- Bitcoin Core: DigitalOcean, Basic, 4 GB, 2 vCPUs ($24/mo)
- chainseeker-syncer: DigitalOcean, Memory-Optimized, 16 GB, 2 vCPUs ($80/mo)
- client: DigitalOcean, CPU-Optimized, 4 GB, 2 vCPUs ($40/mo)

### Results

Date: July 6, 2020 (JST)

#### status
```
┌─────────┬──────┬──────┬───────┬──────┬─────────┬─────────┬───────┐
│ Stat    │ 2.5% │ 50%  │ 97.5% │ 99%  │ Avg     │ Stdev   │ Max   │
├─────────┼──────┼──────┼───────┼──────┼─────────┼─────────┼───────┤
│ Latency │ 0 ms │ 0 ms │ 0 ms  │ 0 ms │ 0.02 ms │ 0.31 ms │ 20 ms │
└─────────┴──────┴──────┴───────┴──────┴─────────┴─────────┴───────┘
┌───────────┬─────────┬─────────┬─────────┬─────────┬──────────┬─────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%     │ 97.5%   │ Avg      │ Stdev   │ Min     │
├───────────┼─────────┼─────────┼─────────┼─────────┼──────────┼─────────┼─────────┤
│ Req/Sec   │ 9695    │ 9695    │ 10671   │ 11063   │ 10622.91 │ 323.18  │ 9693    │
├───────────┼─────────┼─────────┼─────────┼─────────┼──────────┼─────────┼─────────┤
│ Bytes/Sec │ 1.83 MB │ 1.83 MB │ 2.02 MB │ 2.09 MB │ 2.01 MB  │ 60.9 kB │ 1.83 MB │
└───────────┴─────────┴─────────┴─────────┴─────────┴──────────┴─────────┴─────────┘

Req/Bytes counts sampled once per second.

117k requests in 11.02s, 22.1 MB read
```

#### block/500000
```
┌─────────┬──────┬──────┬───────┬──────┬─────────┬─────────┬───────┐
│ Stat    │ 2.5% │ 50%  │ 97.5% │ 99%  │ Avg     │ Stdev   │ Max   │
├─────────┼──────┼──────┼───────┼──────┼─────────┼─────────┼───────┤
│ Latency │ 0 ms │ 0 ms │ 1 ms  │ 1 ms │ 0.07 ms │ 0.37 ms │ 14 ms │
└─────────┴──────┴──────┴───────┴──────┴─────────┴─────────┴───────┘
┌───────────┬─────────┬─────────┬─────────┬─────────┬─────────┬────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%     │ 97.5%   │ Avg     │ Stdev  │ Min     │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼────────┼─────────┤
│ Req/Sec   │ 5183    │ 5183    │ 5731    │ 5911    │ 5730.4  │ 202.77 │ 5181    │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼────────┼─────────┤
│ Bytes/Sec │ 4.29 MB │ 4.29 MB │ 4.75 MB │ 4.89 MB │ 4.74 MB │ 168 kB │ 4.29 MB │
└───────────┴─────────┴─────────┴─────────┴─────────┴─────────┴────────┴─────────┘

Req/Bytes counts sampled once per second.

57k requests in 10.02s, 47.4 MB read
```

#### block_with_txids/500000
```
┌─────────┬──────┬───────┬───────┬───────┬──────────┬─────────┬───────┐
│ Stat    │ 2.5% │ 50%   │ 97.5% │ 99%   │ Avg      │ Stdev   │ Max   │
├─────────┼──────┼───────┼───────┼───────┼──────────┼─────────┼───────┤
│ Latency │ 7 ms │ 12 ms │ 22 ms │ 28 ms │ 12.75 ms │ 4.01 ms │ 42 ms │
└─────────┴──────┴───────┴───────┴───────┴──────────┴─────────┴───────┘
┌───────────┬─────────┬─────────┬─────────┬─────────┬─────────┬────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%     │ 97.5%   │ Avg     │ Stdev  │ Min     │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼────────┼─────────┤
│ Req/Sec   │ 296     │ 296     │ 301     │ 310     │ 301.7   │ 4.01   │ 296     │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼────────┼─────────┤
│ Bytes/Sec │ 53.8 MB │ 53.8 MB │ 54.7 MB │ 56.4 MB │ 54.8 MB │ 725 kB │ 53.8 MB │
└───────────┴─────────┴─────────┴─────────┴─────────┴─────────┴────────┴─────────┘

Req/Bytes counts sampled once per second.

3k requests in 10.02s, 548 MB read
```

#### block_with_txs/500000
```
┌─────────┬─────────┬─────────┬─────────┬─────────┬───────────┬────────────┬─────────┐
│ Stat    │ 2.5%    │ 50%     │ 97.5%   │ 99%     │ Avg       │ Stdev      │ Max     │
├─────────┼─────────┼─────────┼─────────┼─────────┼───────────┼────────────┼─────────┤
│ Latency │ 1794 ms │ 3597 ms │ 7301 ms │ 7301 ms │ 3455.1 ms │ 1730.77 ms │ 7301 ms │
└─────────┴─────────┴─────────┴─────────┴─────────┴───────────┴────────────┴─────────┘
┌───────────┬─────┬──────┬─────┬───────┬─────────┬─────────┬───────┐
│ Stat      │ 1%  │ 2.5% │ 50% │ 97.5% │ Avg     │ Stdev   │ Min   │
├───────────┼─────┼──────┼─────┼───────┼─────────┼─────────┼───────┤
│ Req/Sec   │ 0   │ 0    │ 0   │ 3     │ 1       │ 1.27    │ 2     │
├───────────┼─────┼──────┼─────┼───────┼─────────┼─────────┼───────┤
│ Bytes/Sec │ 0 B │ 0 B  │ 0 B │ 24 MB │ 8.01 MB │ 10.1 MB │ 16 MB │
└───────────┴─────┴──────┴─────┴───────┴─────────┴─────────┴───────┘

Req/Bytes counts sampled once per second.

10 requests in 10.02s, 80.1 MB read
```

#### tx/677b67a894d2587c423976ed65131d5ea730d9bd164e7692beffc0441f40eebf
```
┌─────────┬──────┬──────┬───────┬──────┬─────────┬─────────┬───────┐
│ Stat    │ 2.5% │ 50%  │ 97.5% │ 99%  │ Avg     │ Stdev   │ Max   │
├─────────┼──────┼──────┼───────┼──────┼─────────┼─────────┼───────┤
│ Latency │ 0 ms │ 1 ms │ 2 ms  │ 2 ms │ 0.98 ms │ 0.48 ms │ 18 ms │
└─────────┴──────┴──────┴───────┴──────┴─────────┴─────────┴───────┘
┌───────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%     │ 97.5%   │ Avg     │ Stdev   │ Min     │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
│ Req/Sec   │ 3159    │ 3159    │ 3333    │ 3417    │ 3333.19 │ 68.67   │ 3158    │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
│ Bytes/Sec │ 46.6 MB │ 46.6 MB │ 49.2 MB │ 50.5 MB │ 49.2 MB │ 1.02 MB │ 46.6 MB │
└───────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────┘

Req/Bytes counts sampled once per second.

37k requests in 11.02s, 541 MB read
```

#### txids/bc1qgdjqv0av3q56jvd82tkdjpy7gdp9ut8tlqmgrpmv24sq90ecnvqqjwvw97
```
┌─────────┬──────┬──────┬───────┬──────┬─────────┬────────┬───────┐
│ Stat    │ 2.5% │ 50%  │ 97.5% │ 99%  │ Avg     │ Stdev  │ Max   │
├─────────┼──────┼──────┼───────┼──────┼─────────┼────────┼───────┤
│ Latency │ 0 ms │ 0 ms │ 1 ms  │ 2 ms │ 0.19 ms │ 0.5 ms │ 12 ms │
└─────────┴──────┴──────┴───────┴──────┴─────────┴────────┴───────┘
┌───────────┬─────────┬─────────┬─────────┬─────────┬─────────┬────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%     │ 97.5%   │ Avg     │ Stdev  │ Min     │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼────────┼─────────┤
│ Req/Sec   │ 3919    │ 3919    │ 4331    │ 4663    │ 4349.3  │ 210.94 │ 3919    │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼────────┼─────────┤
│ Bytes/Sec │ 18.5 MB │ 18.5 MB │ 20.5 MB │ 22.1 MB │ 20.6 MB │ 998 kB │ 18.5 MB │
└───────────┴─────────┴─────────┴─────────┴─────────┴─────────┴────────┴─────────┘

Req/Bytes counts sampled once per second.

43k requests in 10.02s, 206 MB read
```

#### txs/bc1qgdjqv0av3q56jvd82tkdjpy7gdp9ut8tlqmgrpmv24sq90ecnvqqjwvw97
```
┌─────────┬───────┬───────┬────────┬────────┬──────────┬─────────┬────────┐
│ Stat    │ 2.5%  │ 50%   │ 97.5%  │ 99%    │ Avg      │ Stdev   │ Max    │
├─────────┼───────┼───────┼────────┼────────┼──────────┼─────────┼────────┤
│ Latency │ 59 ms │ 75 ms │ 122 ms │ 146 ms │ 80.72 ms │ 25.1 ms │ 318 ms │
└─────────┴───────┴───────┴────────┴────────┴──────────┴─────────┴────────┘
┌───────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%     │ 97.5%   │ Avg     │ Stdev   │ Min     │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
│ Req/Sec   │ 35      │ 35      │ 50      │ 53      │ 49      │ 4.94    │ 35      │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
│ Bytes/Sec │ 22.1 MB │ 22.1 MB │ 31.5 MB │ 33.4 MB │ 30.9 MB │ 3.12 MB │ 22.1 MB │
└───────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────┘

Req/Bytes counts sampled once per second.

490 requests in 10.05s, 309 MB read
```

#### utxos/bc1qgdjqv0av3q56jvd82tkdjpy7gdp9ut8tlqmgrpmv24sq90ecnvqqjwvw97
```
┌─────────┬──────┬──────┬───────┬──────┬─────────┬────────┬───────┐
│ Stat    │ 2.5% │ 50%  │ 97.5% │ 99%  │ Avg     │ Stdev  │ Max   │
├─────────┼──────┼──────┼───────┼──────┼─────────┼────────┼───────┤
│ Latency │ 0 ms │ 0 ms │ 0 ms  │ 1 ms │ 0.03 ms │ 0.3 ms │ 19 ms │
└─────────┴──────┴──────┴───────┴──────┴─────────┴────────┴───────┘
┌───────────┬────────┬────────┬─────────┬─────────┬─────────┬────────┬────────┐
│ Stat      │ 1%     │ 2.5%   │ 50%     │ 97.5%   │ Avg     │ Stdev  │ Min    │
├───────────┼────────┼────────┼─────────┼─────────┼─────────┼────────┼────────┤
│ Req/Sec   │ 7167   │ 7167   │ 7727    │ 7911    │ 7703.46 │ 184.69 │ 7165   │
├───────────┼────────┼────────┼─────────┼─────────┼─────────┼────────┼────────┤
│ Bytes/Sec │ 4.1 MB │ 4.1 MB │ 4.42 MB │ 4.53 MB │ 4.41 MB │ 106 kB │ 4.1 MB │
└───────────┴────────┴────────┴─────────┴─────────┴─────────┴────────┴────────┘

Req/Bytes counts sampled once per second.

85k requests in 11.02s, 48.5 MB read
```

#### rich_list_count
```
┌─────────┬──────┬──────┬───────┬──────┬─────────┬────────┬───────┐
│ Stat    │ 2.5% │ 50%  │ 97.5% │ 99%  │ Avg     │ Stdev  │ Max   │
├─────────┼──────┼──────┼───────┼──────┼─────────┼────────┼───────┤
│ Latency │ 0 ms │ 0 ms │ 0 ms  │ 0 ms │ 0.02 ms │ 0.4 ms │ 23 ms │
└─────────┴──────┴──────┴───────┴──────┴─────────┴────────┴───────┘
┌───────────┬─────────┬─────────┬─────────┬─────────┬──────────┬────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%     │ 97.5%   │ Avg      │ Stdev  │ Min     │
├───────────┼─────────┼─────────┼─────────┼─────────┼──────────┼────────┼─────────┤
│ Req/Sec   │ 9135    │ 9135    │ 10647   │ 10911   │ 10498.55 │ 509.83 │ 9128    │
├───────────┼─────────┼─────────┼─────────┼─────────┼──────────┼────────┼─────────┤
│ Bytes/Sec │ 1.73 MB │ 1.73 MB │ 2.02 MB │ 2.07 MB │ 1.99 MB  │ 97 kB  │ 1.73 MB │
└───────────┴─────────┴─────────┴─────────┴─────────┴──────────┴────────┴─────────┘

Req/Bytes counts sampled once per second.

115k requests in 11.01s, 21.9 MB read
```

#### rich_list/0/100
```
┌─────────┬──────┬──────┬───────┬──────┬─────────┬─────────┬───────┐
│ Stat    │ 2.5% │ 50%  │ 97.5% │ 99%  │ Avg     │ Stdev   │ Max   │
├─────────┼──────┼──────┼───────┼──────┼─────────┼─────────┼───────┤
│ Latency │ 1 ms │ 1 ms │ 3 ms  │ 4 ms │ 1.49 ms │ 0.78 ms │ 17 ms │
└─────────┴──────┴──────┴───────┴──────┴─────────┴─────────┴───────┘
┌───────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%     │ 97.5%   │ Avg     │ Stdev   │ Min     │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
│ Req/Sec   │ 1854    │ 1854    │ 2042    │ 2093    │ 2029.91 │ 60.51   │ 1854    │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
│ Bytes/Sec │ 49.7 MB │ 49.7 MB │ 54.8 MB │ 56.1 MB │ 54.4 MB │ 1.61 MB │ 49.7 MB │
└───────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────┘

Req/Bytes counts sampled once per second.

22k requests in 11.02s, 599 MB read
```

#### rich_list_addr_rank/bc1qgdjqv0av3q56jvd82tkdjpy7gdp9ut8tlqmgrpmv24sq90ecnvqqjwvw97
```
┌─────────┬──────┬──────┬───────┬──────┬─────────┬─────────┬───────┐
│ Stat    │ 2.5% │ 50%  │ 97.5% │ 99%  │ Avg     │ Stdev   │ Max   │
├─────────┼──────┼──────┼───────┼──────┼─────────┼─────────┼───────┤
│ Latency │ 0 ms │ 0 ms │ 0 ms  │ 0 ms │ 0.02 ms │ 0.52 ms │ 29 ms │
└─────────┴──────┴──────┴───────┴──────┴─────────┴─────────┴───────┘
┌───────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%     │ 97.5%   │ Avg     │ Stdev   │ Min     │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
│ Req/Sec   │ 8567    │ 8567    │ 9767    │ 9927    │ 9637.46 │ 359.49  │ 8563    │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤
│ Bytes/Sec │ 1.56 MB │ 1.56 MB │ 1.78 MB │ 1.81 MB │ 1.75 MB │ 65.6 kB │ 1.56 MB │
└───────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────┴─────────┘

Req/Bytes counts sampled once per second.

106k requests in 11.02s, 19.3 MB read
```

### Results for Bitcoin Core REST API (just for comparison)

Date: July 5, 2020 (JST)

#### chaininfo.json
```
┌─────────┬──────┬──────┬───────┬──────┬─────────┬─────────┬───────┐
│ Stat    │ 2.5% │ 50%  │ 97.5% │ 99%  │ Avg     │ Stdev   │ Max   │
├─────────┼──────┼──────┼───────┼──────┼─────────┼─────────┼───────┤
│ Latency │ 1 ms │ 2 ms │ 4 ms  │ 5 ms │ 1.84 ms │ 0.91 ms │ 18 ms │
└─────────┴──────┴──────┴───────┴──────┴─────────┴─────────┴───────┘
┌───────────┬─────────┬─────────┬─────────┬─────────┬─────────┬────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%     │ 97.5%   │ Avg     │ Stdev  │ Min     │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼────────┼─────────┤
│ Req/Sec   │ 6255    │ 6255    │ 6819    │ 7579    │ 6872.4  │ 353.18 │ 6253    │
├───────────┼─────────┼─────────┼─────────┼─────────┼─────────┼────────┼─────────┤
│ Bytes/Sec │ 5.99 MB │ 5.99 MB │ 6.54 MB │ 7.27 MB │ 6.59 MB │ 340 kB │ 5.99 MB │
└───────────┴─────────┴─────────┴─────────┴─────────┴─────────┴────────┴─────────┘

Req/Bytes counts sampled once per second.

69k requests in 10.02s, 65.9 MB read
```

#### blockhashbyheight/500000.json
```
┌─────────┬──────┬──────┬───────┬──────┬─────────┬─────────┬──────┐
│ Stat    │ 2.5% │ 50%  │ 97.5% │ 99%  │ Avg     │ Stdev   │ Max  │
├─────────┼──────┼──────┼───────┼──────┼─────────┼─────────┼──────┤
│ Latency │ 0 ms │ 1 ms │ 1 ms  │ 2 ms │ 0.88 ms │ 0.46 ms │ 9 ms │
└─────────┴──────┴──────┴───────┴──────┴─────────┴─────────┴──────┘
┌───────────┬─────────┬─────────┬─────────┬─────────┬──────────┬────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%     │ 97.5%   │ Avg      │ Stdev  │ Min     │
├───────────┼─────────┼─────────┼─────────┼─────────┼──────────┼────────┼─────────┤
│ Req/Sec   │ 10943   │ 10943   │ 12095   │ 14439   │ 12297.82 │ 991.01 │ 10941   │
├───────────┼─────────┼─────────┼─────────┼─────────┼──────────┼────────┼─────────┤
│ Bytes/Sec │ 2.07 MB │ 2.07 MB │ 2.29 MB │ 2.73 MB │ 2.32 MB  │ 187 kB │ 2.07 MB │
└───────────┴─────────┴─────────┴─────────┴─────────┴──────────┴────────┴─────────┘

Req/Bytes counts sampled once per second.

135k requests in 11.02s, 25.6 MB read
```

#### block/00000000000000000024fb37364cbf81fd49cc2d51c09c75c35433c3a1945d04.json
```
┌─────────┬─────────┬─────────┬─────────┬─────────┬────────────┬───────────┬─────────┐
│ Stat    │ 2.5%    │ 50%     │ 97.5%   │ 99%     │ Avg        │ Stdev     │ Max     │
├─────────┼─────────┼─────────┼─────────┼─────────┼────────────┼───────────┼─────────┤
│ Latency │ 1840 ms │ 2118 ms │ 2821 ms │ 2930 ms │ 2128.19 ms │ 224.77 ms │ 2930 ms │
└─────────┴─────────┴─────────┴─────────┴─────────┴────────────┴───────────┴─────────┘
┌───────────┬─────┬──────┬─────────┬─────────┬─────────┬─────────┬─────────┐
│ Stat      │ 1%  │ 2.5% │ 50%     │ 97.5%   │ Avg     │ Stdev   │ Min     │
├───────────┼─────┼──────┼─────────┼─────────┼─────────┼─────────┼─────────┤
│ Req/Sec   │ 0   │ 0    │ 4       │ 13      │ 6.6     │ 4.5     │ 2       │
├───────────┼─────┼──────┼─────────┼─────────┼─────────┼─────────┼─────────┤
│ Bytes/Sec │ 0 B │ 0 B  │ 29.8 MB │ 96.7 MB │ 49.1 MB │ 33.5 MB │ 14.9 MB │
└───────────┴─────┴──────┴─────────┴─────────┴─────────┴─────────┴─────────┘

Req/Bytes counts sampled once per second.

66 requests in 10.03s, 491 MB read
```

#### tx/677b67a894d2587c423976ed65131d5ea730d9bd164e7692beffc0441f40eebf.json
```
┌─────────┬──────┬──────┬───────┬───────┬─────────┬─────────┬───────┐
│ Stat    │ 2.5% │ 50%  │ 97.5% │ 99%   │ Avg     │ Stdev   │ Max   │
├─────────┼──────┼──────┼───────┼───────┼─────────┼─────────┼───────┤
│ Latency │ 1 ms │ 3 ms │ 8 ms  │ 11 ms │ 3.52 ms │ 2.49 ms │ 64 ms │
└─────────┴──────┴──────┴───────┴───────┴─────────┴─────────┴───────┘
┌───────────┬─────────┬─────────┬───────┬─────────┬─────────┬─────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%   │ 97.5%   │ Avg     │ Stdev   │ Min     │
├───────────┼─────────┼─────────┼───────┼─────────┼─────────┼─────────┼─────────┤
│ Req/Sec   │ 2173    │ 2173    │ 4179  │ 4311    │ 3984.4  │ 611     │ 2172    │
├───────────┼─────────┼─────────┼───────┼─────────┼─────────┼─────────┼─────────┤
│ Bytes/Sec │ 28.6 MB │ 28.6 MB │ 55 MB │ 56.7 MB │ 52.4 MB │ 8.05 MB │ 28.6 MB │
└───────────┴─────────┴─────────┴───────┴─────────┴─────────┴─────────┴─────────┘

Req/Bytes counts sampled once per second.

40k requests in 10.02s, 524 MB read
```

