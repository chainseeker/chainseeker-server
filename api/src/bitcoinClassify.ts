
interface BitcoinClassify {
	output(script: Buffer): string;
	input(script: Buffer, allowIncomplete: boolean): string;
	witness(script: Buffer[], allowIncomplete: boolean): string;
}

const bitcoinClassify = require('bitcoinjs-lib/src/classify') as BitcoinClassify;

export default bitcoinClassify;

