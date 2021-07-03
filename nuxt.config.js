
export default {
	
	ssr: false,
	
	// Target: https://go.nuxtjs.dev/config-target
	target: 'static',
	
	// Global page headers: https://go.nuxtjs.dev/config-head
	head: {
		title: 'chainseeker - an open-source block explorer',
	},
	
	// Global CSS: https://go.nuxtjs.dev/config-css
	css: [
	],
	
	// Plugins to run before rendering page: https://go.nuxtjs.dev/config-plugins
	plugins: [
	],
	
	// Auto import components: https://go.nuxtjs.dev/config-components
	components: true,
	
	// Modules for dev and build (recommended): https://go.nuxtjs.dev/config-modules
	buildModules: [
		// https://go.nuxtjs.dev/typescript
		'@nuxt/typescript-build',
		'@nuxtjs/vuetify',
	],
	
	// Modules: https://go.nuxtjs.dev/config-modules
	modules: [
	],
	
	// Build Configuration: https://go.nuxtjs.dev/config-build
	build: {
	},
	
	publicRuntimeConfig: {
		coins: {
			btc: {
				coin: {
					name: "Bitcoin (Mainnet)",
					symbol: "BTC",
					satoshi: "satoshi",
					blockReward: {
						initial: 5000000000,
						halving: 210000
					},
				},
				apiEndpoint: 'https://btc-v3.chainseeker.info/api',
				wsEndpoint: 'wss://btc-v3.chainseeker.info/ws',
			},
			tbtc: {
				coin: {
					name: "Bitcoin (Testnet)",
					symbol: "tBTC",
					satoshi: "satoshi",
					blockReward: {
						initial: 5000000000,
						halving: 210000
					},
				},
				apiEndpoint: 'https://tbtc-v3.chainseeker.info/api',
				wsEndpoint: 'wss://tbtc-v3.chainseeker.info/ws',
			},
			sbtc: {
				coin: {
					name: "Bitcoin (Signet)",
					symbol: "sBTC",
					satoshi: "satoshi",
					blockReward: {
						initial: 5000000000,
						halving: 210000
					},
				},
				apiEndpoint: 'https://sbtc-v3.chainseeker.info/api',
				wsEndpoint: 'wss://sbtc-v3.chainseeker.info/ws',
			},
			mona: {
				coin: {
					name: "Monacoin",
					symbol: "MONA",
					satoshi: "watanabe",
					blockReward: {
						initial: 5000000000,
						halving: 1051200
					},
				},
				apiEndpoint: 'https://mona-v3.chainseeker.info/api',
				wsEndpoint: 'wss://mona-v3.chainseeker.info/ws',
			},
			local: {
				coin: {
					name: "Bitcoin (Local)",
					symbol: "lBTC",
					satoshi: "satoshi",
					blockReward: {
						initial: 5000000000,
						halving: 210000
					},
				},
				apiEndpoint: 'http://localhost:8000/api',
				wsEndpoint: 'ws://localhost:8001',
			},
		}
	},
	
	generate: {
		fallback: true,
	},
	
};

