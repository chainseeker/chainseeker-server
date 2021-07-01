
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
				},
				apiEndpoint: 'https://btc.chainseeker.info/api',
				wsEndpoint: 'wss://btc.chainseeker.info/ws',
			},
			tbtc: {
				coin: {
					name: "Bitcoin (Testnet)",
					symbol: "tBTC",
					satoshi: "satoshi",
				},
				apiEndpoint: 'https://tbtc.chainseeker.info/api',
				wsEndpoint: 'wss://tbtc.chainseeker.info/ws',
			},
			local: {
				coin: {
					name: "Bitcoin (Local)",
					symbol: "lBTC",
					satoshi: "satoshi",
				},
				apiEndpoint: 'http://localhost:6001/api',
				wsEndpoint: 'ws://localhost:7001',
			},
		}
	},
	
	generate: {
		fallback: true,
	},
	
};

