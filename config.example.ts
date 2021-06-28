
export const configs = {
	btc: {
		google_analytics: "UA-000000000-0",
		coin: {
			network: "bitcoin",
			name: "Bitcoin",
			symbol: "BTC",
			satoshi: "satoshi",
			block_reward: {
				initial: 5000000000,
				halving: 210000,
			},
		},
		coind: {
			user: "bitcoin",
			pass: "bitcoinrpc",
			port: 8332,
			host: "localhost",
			zmq: "tcp://127.0.0.1:28332"
		},
		rest: {
			endpoint: 'http://localhost:8332/rest',
		},
		syncer: {
			endpoint: 'http://localhost:8080',
		},
		endpoint: {
			apiLocal: "http://localhost:7001/api",
			apiPublic: "http://localhost:7001/api",
			webPublic: "http://localhost:7002",
			websocket: "ws://localhost:7001/ws",
		},
		server: {
			api: {
				host: "0.0.0.0",
				port: 7001,
				prefix: "/api",
			},
			web: {
				host: "0.0.0.0",
				port: 7002,
				prefix: "",
			}
		},
		debug: false,
	},
};

