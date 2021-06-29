
import * as bitcoin from 'bitcoinjs-lib';

import * as cs from 'chainseeker/dist/types';

import { SyncerClient } from './SyncerClient';

export const resolveAddress = (script: Buffer, network: bitcoin.Network): string|null => {
	try {
		return bitcoin.address.fromOutputScript(script, network);
	} catch(e) {
		return null;
	}
};

export const fetchTransaction = async (syncer: SyncerClient, txhash: string):
	Promise<cs.Transaction> => {
	return await syncer.getTx(txhash);
}

export const fetchBlockByHeight = async (syncer: SyncerClient, height: number): Promise<cs.Block> => {
	return await syncer.getBlockByHeight(height);
};

export const fetchBlockByHash = async (syncer: SyncerClient, hash: string): Promise<cs.Block> => {
	return await syncer.getBlock(hash);
};

