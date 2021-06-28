
import * as bitcoin from 'bitcoinjs-lib';
import bitcoinClassify from './bitcoinClassify';
import * as xcp from 'counterjs';

import * as cs from 'chainseeker/dist/types';

import { RestClient } from './RestClient';
import { SyncerClient, SyncerBlock } from './SyncerClient';

export const resolveAddress = (script: Buffer, network: bitcoin.Network): string|null => {
	try {
		return bitcoin.address.fromOutputScript(script, network);
	} catch(e) {
		return null;
	}
};

export type RawTransaction = {
	confirmed_height: number;
	rawtx: Buffer;
};

const fetchTransactionWithConfirmedHeight = async (syncer: SyncerClient, rest: RestClient, txhash: string):
	Promise<{ tx: bitcoin.Transaction, confirmed_height: number | null }> => {
	const restTx = await rest.getTxJson(txhash);
	const tx = bitcoin.Transaction.fromBuffer(Buffer.from(restTx.hex, 'hex'));
	if(restTx.blockhash) {
		const syncerBlock = await syncer.getBlock(restTx.blockhash);
		return {
			confirmed_height: syncerBlock.height,
			tx,
		};
	} else {
		return {
			confirmed_height: null,
			tx,
		};
	}
};

export const fetchTransaction = async (syncer: SyncerClient, rest: RestClient, txhash: string, network: bitcoin.Network):
	Promise<cs.Transaction> => {
	const { confirmed_height, tx } = await fetchTransactionWithConfirmedHeight(syncer, rest, txhash);
	// Fetch input tx information.
	let isCoinbase = false;
	const previousTxs = await Promise.all(tx.ins.map((input) => {
		const txhash = Buffer.from(input.hash);
		// Coinbase transaction.
		if(txhash.toString('hex') == '0000000000000000000000000000000000000000000000000000000000000000') {
			isCoinbase = true;
			return undefined;
		}
		// Fetch raw transaction for this input.
		return rest.getTx((txhash.reverse() as Buffer).toString('hex'))
	}));
	// Compute transaction fee.
	const inputValue = tx.ins.reduce((acc, input, i) => {
		const previousTx = previousTxs[i];
		const val = (typeof previousTx === 'undefined' ? 0 : previousTx.outs[input.index].value);
		return acc + val;
	}, 0);
	const outputValue = tx.outs.reduce((acc, output) => acc + output.value, 0);
	const fee = inputValue - outputValue;
	// Parse Counterparty.
	let counterparty = undefined;
	try {
		const parsed = xcp.util.parseTransaction(tx.toBuffer());
		counterparty = {
			destination: parsed.destination,
			message: parsed.message.toJSON(),
		};
	} catch(e) {}
	// Convert to cs.Transaction.
	return {
		hex: tx.toBuffer().toString('hex'),
		txid: tx.getId(),
		hash: (bitcoin.crypto.hash256(tx.toBuffer()).reverse() as Buffer).toString('hex'),
		size: tx.byteLength(),
		vsize: tx.virtualSize(),
		version: tx.version,
		locktime: tx.locktime,
		confirmed_height,
		counterparty: counterparty,
		vin: tx.ins.map((input: bitcoin.TxInput) => ({
			txid: (Buffer.from(input.hash).reverse() as Buffer).toString('hex'),
			vout: input.index,
			scriptSig: {
				asm: isCoinbase ? '' : bitcoin.script.toASM(input.script),
				hex: input.script.toString('hex'),
			},
			txinwitness: input.witness.map((witness: Buffer) => witness.toString('hex')),
			sequence: input.sequence,
			value: (<any>input).previousTx ? (<any>input).previousTx.outs[input.index].value : 0,
			address: (<any>input).previousTx ? resolveAddress((<any>input).previousTx.outs[input.index].script, network) : 'coinbase',
		})),
		vout: (<bitcoin.TxOutput[]>tx.outs).map((output: bitcoin.TxOutput, n: number) => ({
			value: output.value,
			n: n,
			scriptPubKey: {
				asm: bitcoin.script.toASM(output.script),
				hex: output.script.toString('hex'),
				type: bitcoinClassify.output(output.script),
				address: resolveAddress(output.script, network),
			},
		})),
		fee: fee,
	};
}

type RawBlockHeader = {
	header: Buffer;
	size: number;
	strippedsize: number;
	weight: number;
	txhashes: Buffer[];
};

const convertBlock = (syncerBlock: SyncerBlock): cs.Block => {
	const block = bitcoin.Block.fromHex(syncerBlock.block_header);
	return {
		header: syncerBlock.block_header,
		hash: block.getId(),
		version: (<any>block).version,
		previousblockhash: ((<any>block).prevHash.reverse() as Buffer).toString('hex'),
		merkleroot: ((<any>block).merkleRoot.reverse() as Buffer).toString('hex'),
		time: (<any>block).timestamp,
		bits: (<any>block).bits.toString(16),
		difficulty: (Math.pow(2., 8 * (0x1d - ((<any>block).bits>>24))) * 0x00ffff / ((<any>block).bits & 0x00ffffff)),
		nonce: (<any>block).nonce,
		size:         syncerBlock.size,
		strippedsize: syncerBlock.strippedsize,
		weight:       syncerBlock.weight,
		height:       syncerBlock.height,
		txids:        syncerBlock.txids,
	};
};

export const fetchBlockByHeight = async (syncer: SyncerClient, height: number): Promise<cs.Block> => {
	return convertBlock(await syncer.getBlockByHeight(height));
};

export const fetchBlockByHash = async (syncer: SyncerClient, hash: string): Promise<cs.Block> => {
	return convertBlock(await syncer.getBlock(hash));
};

