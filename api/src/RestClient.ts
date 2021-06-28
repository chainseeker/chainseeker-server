
import fetch from 'node-fetch';
import * as bitcoin from 'bitcoinjs-lib';
import { Semaphore } from 'await-semaphore';

export type RestVin = {
	coinbase?: string,
	txinwitness: string[],
	sequence: number,
};

export type RestVout = {
	value: number,
	n: number,
	scriptPubKey: {
		asm: string,
		hex: string,
		reqSigs?: number,
		type: string,
		addresses?: string[],
	}
};

export type RestTxInBlock = {
	txid: string,
	hash: string,
	version: number,
	size: number,
	vsize: number,
	weight: number,
	locktime: number,
	vin: RestVin[],
	vout: RestVout[],
};

export type RestBlock = {
	hash: string,
	confirmations: number,
	strippedsize: number;
	size: number;
	weight: number;
	height: number,
	version: number,
	merkleroot: string,
	tx: RestTxInBlock[],
	time: number,
	mediantime: number,
	nonce: number,
	bits: string,
	difficulty: number,
	chainwork: string,
	nTx: number,
	previousblockhash: string,
	nextblockhash?: string,
};

export type RestTx = {
	blockhash?: string,
	hex: string,
};

export type RestChainInfo = {
	blocks: number,
};

export class RestClient {
	private semaphore = new Semaphore(4);
	constructor(private endpoint: string) {
	}
	private async callJson<T>(...path: string[]): Promise<T> {
		const url = `${this.endpoint}/${path.join('/')}.json`;
		const release = await this.semaphore.acquire();
		const response = await fetch(url);
		const json = await response.json() as T;
		release();
		return json;
	}
	private async callBin(...path: string[]): Promise<Buffer> {
		const url = `${this.endpoint}/${path.join('/')}.bin`;
		const release = await this.semaphore.acquire();
		const response = await fetch(url);
		const buf = await response.buffer();
		release();
		return buf;
	}
	async getBlockJson(blockid: string): Promise<RestBlock> {
		return await this.callJson<RestBlock>('block', blockid);
	}
	async getBlockBin(blockid: string): Promise<Buffer> {
		return await this.callBin('block', blockid);
	}
	async getBlockNoTxDetailsJson(blockid: string): Promise<RestBlock> {
		return await this.callJson<RestBlock>('block', 'notxdetails', blockid);
	}
	async getBlockNoTxDetailsBin(blockid: string): Promise<Buffer> {
		return await this.callBin('block', 'notxdetails', blockid);
	}
	async getTxJson(txid: string): Promise<RestTx> {
		return await this.callJson<RestTx>('tx', txid);
	}
	async getTxBin(txid: string): Promise<Buffer> {
		return await this.callBin('tx', txid);
	}
	async getTx(txid: string): Promise<bitcoin.Transaction> {
		const rawtx = await this.getTxBin(txid);
		return bitcoin.Transaction.fromBuffer(rawtx);
	}
	async getBlockHashByHeightBin(height: number): Promise<Buffer> {
		return await this.callBin('blockhashbyheight', height.toString());
	}
	async getChainInfo(): Promise<RestChainInfo> {
		return await this.callJson<RestChainInfo>('chaininfo');
	}
}

