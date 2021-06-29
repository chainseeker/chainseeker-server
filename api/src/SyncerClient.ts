
import fetch from 'node-fetch';
import * as cs from 'chainseeker/dist/types';

export type SyncerAddressIndex = string[];
export type SyncerUtxoEntry = {
	txid: string,
	vout: number,
	value: number,
};
export type SyncerUtxo = SyncerUtxoEntry[];
export type SyncerRichListEntry = {
	script_pubkey: string,
	value: number,
};
export type SyncerRichList = SyncerRichListEntry[];
export type SyncerBlockSummary = {
	hash: string,
	time: number,
	nonce: number,
	size: number,
	strippedsize: number,
	weight: number,
	txcount: number,
};

export class SyncerClient {
	constructor(private endpoint: string) {
	}
	private async call<T>(...path: string[]): Promise<T> {
		const url = `${this.endpoint}/api/v1/${path.join('/')}`;
		const response = await fetch(url);
		const json = await response.json() as T;
		return json;
	}
	async getStatus(): Promise<cs.Status> {
		return await this.call<cs.Status>('status');
	}
	async getBlockSummary(offset: number, limit: number): Promise<SyncerBlockSummary> {
		return await this.call<SyncerBlockSummary>('block_summary', offset.toString(), limit.toString());
	}
	async getBlock(param: string | number): Promise<cs.Block> {
		return await this.call<cs.Block>('block', typeof param == 'number' ? param.toString() : param);
	}
	async getTx(txhash: string): Promise<cs.Transaction> {
		return await this.call<cs.Transaction>('tx', txhash);
	}
	async getAddressIndex(script: Buffer): Promise<SyncerAddressIndex> {
		return await this.call<SyncerAddressIndex>('addr_index', script.toString('hex'));
	}
	async getUtxo(script: Buffer): Promise<SyncerUtxo> {
		return await this.call<SyncerUtxo>('utxo', script.toString('hex'));
	}
	async getRichListCount(): Promise<number> {
		return (await this.call<{count: number}>('rich_list', 'count')).count;
	}
	async getRichList(offset: number, limit: number): Promise<SyncerRichList> {
		return await this.call<SyncerRichList>('rich_list', offset.toString(), limit.toString());
	}
}

