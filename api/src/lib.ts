
import * as bitcoin from 'bitcoinjs-lib';
import bitcoinClassify from './bitcoinClassify';
import * as xcp from 'counterjs';

import * as cs from 'chainseeker/dist/types';

import { RestClient } from './RestClient';

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

export const fetchRawTransaction = async (rest: RestClient, txhash: string): Promise<RawTransaction> => {
	const restTx = await rest.getTxJson(txhash);
	if(restTx.blockhash) {
		const restBlock = await rest.getBlockJson(restTx.blockhash);
		return {
			confirmed_height: restBlock.height,
			rawtx: Buffer.from(restTx.hex, 'hex'),
		};
	} else {
		return {
			confirmed_height: -1,
			rawtx: Buffer.from(restTx.hex, 'hex'),
		};
	}
}

export const fetchTransaction = async (rest: RestClient, txhash: string, network: bitcoin.Network): Promise<cs.Transaction> => {
	const data = await fetchRawTransaction(rest, txhash);
	const tx = bitcoin.Transaction.fromBuffer(data.rawtx);
	const height = data.confirmed_height < 0 ? null : data.confirmed_height;
	// Fetch input tx information.
	let isCoinbase = false;
	for(let i in tx.ins) {
		const input = tx.ins[i];
		const txhash = Buffer.from(input.hash);
		// Coinbase transaction.
		if(txhash.toString('hex') == '0000000000000000000000000000000000000000000000000000000000000000') {
			isCoinbase = true;
			continue;
		}
		// Fetch raw transaction for this input.
		const txInput = await fetchRawTransaction(rest, (txhash.reverse() as Buffer).toString('hex'));
		(<any>tx.ins[i]).previousTx = {
			tx: bitcoin.Transaction.fromBuffer(txInput.rawtx),
			height: (txInput.confirmed_height < 0 ? null : txInput.confirmed_height),
		};
	}
	// Compute transaction fee.
	let fee = 0;
	tx.ins.forEach((input) => { fee += (<any>input).previousTx ? (<any>input).previousTx.tx.outs[input.index].value : 0; });
	(<bitcoin.TxOutput[]>tx.outs).forEach((output) => { fee -= output.value; });
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
		confirmed_height: height,
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
			value: (<any>input).previousTx ? (<any>input).previousTx.tx.outs[input.index].value : 0,
			address: (<any>input).previousTx ? resolveAddress((<any>input).previousTx.tx.outs[input.index].script, network) : 'coinbase',
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

const _fetchRawBlockHeader = async (rest: RestClient, hash: string): Promise<RawBlockHeader> => {
	const restBlockJson = await rest.getBlockJson(hash);
	const restBlockBin = await rest.getBlockBin(hash);
	const block = bitcoin.Block.fromBuffer(restBlockBin);
	return {
		header      : block.toBuffer(true),
		size        : restBlockJson.size,
		strippedsize: restBlockJson.strippedsize,
		weight      : restBlockJson.weight,
		txhashes    : block.transactions!.map((tx) => Buffer.from(tx.getId(), 'hex')),
	};
};

const _fetchBlock = async (rest: RestClient, hash: string, height: number): Promise<cs.Block> => {
	// Get block header.
	const rawBlockHeader: RawBlockHeader = await _fetchRawBlockHeader(rest, hash);
	const block = bitcoin.Block.fromBuffer(rawBlockHeader.header);
	// Construct final object.
	const ret = {
		header: rawBlockHeader.header.toString('hex'),
		hash: block.getId(),
		version: (<any>block).version,
		previousblockhash: ((<any>block).prevHash.reverse() as Buffer).toString('hex'),
		merkleroot: ((<any>block).merkleRoot.reverse() as Buffer).toString('hex'),
		time: (<any>block).timestamp,
		bits: (<any>block).bits.toString(16),
		difficulty: (Math.pow(2., 8 * (0x1d - ((<any>block).bits>>24))) * 0x00ffff / ((<any>block).bits & 0x00ffffff)),
		nonce: (<any>block).nonce,
		size:         rawBlockHeader.size,
		strippedsize: rawBlockHeader.strippedsize,
		weight:       rawBlockHeader.weight,
		height: height,
		txids: rawBlockHeader.txhashes.map((txhash) => txhash.toString('hex')),
	};
	return ret;
}

export const fetchBlockByHeight = async (rest: RestClient, height: number): Promise<cs.Block> => {
	const blockhash = await rest.getBlockHashByHeightBin(height);
	return _fetchBlock(rest, (blockhash.reverse() as Buffer).toString('hex'), height);
};

export const fetchBlockByHash = async (rest: RestClient, hash: string): Promise<cs.Block> => {
	const restBlock = await rest.getBlockNoTxDetailsJson(hash);
	const height = restBlock.height;
	return _fetchBlock(rest, hash, height);
};

