
declare module 'coininfo' {
	interface Bip32 {
		public: number;
		private: number;
	}
	export interface Network {
		messagePrefix: string;
		bech32: string;
		bip32: Bip32;
		pubKeyHash: number;
		scriptHash: number;
		wif: number;
	}
	export default function coininfo(input: string): { toBitcoinJS: () => Network }|null;
}

