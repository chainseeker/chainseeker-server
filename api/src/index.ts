
import express from 'express';
import express_graphql from 'express-graphql';
import { buildSchema } from 'graphql';
import * as bodyParser from 'body-parser';
import compression from 'compression';
import morgan from 'morgan';
import * as bitcoin from 'bitcoinjs-lib';
import bitcoinClassify from './bitcoinClassify';
import * as rpc from 'json-rpc2';
import coininfo from 'coininfo';

import * as cs from 'chainseeker/dist/types';

import { SyncerClient } from './SyncerClient';
import { fetchBlockByHeight, fetchBlockByHash, fetchTransaction, resolveAddress } from './lib';
import WebSocketRelay from './WebSocketRelay';
import GQL_SCHEMA from './gql-schema';

const main = async () => {
	
	const COIN_NAME = process.argv[2];
	const configs = (await import(`${process.env['HOME']}/.chainseeker/config`)).configs;
	const config = configs[COIN_NAME];
	
	const API_VERSION = 'v1';
	
	const COIN = coininfo(config.coin.network as string);
	if(COIN === null) {
		throw new Error('Coin not found.');
	}
	const NETWORK = COIN.toBitcoinJS();
	
	process.on('unhandledRejection', console.dir);
	
	const syncer = new SyncerClient(config.syncer.endpoint);
	const rpcClient = rpc.Client.$create(config.coind.port, config.coind.host, config.coind.user, config.coind.pass);
	
	const app: express.Application = express();
	app.use(morgan('dev'));
	app.use(compression({
		filter: (req, res) => {
			return true;
		},
	}));
	app.use(bodyParser.text({ type: 'text/plain' }));
	app.use(bodyParser.json({ type: 'application/json', limit: '1mb' }));
	
	// Allow CORS.
	app.use((req, res, next) => {
		res.header('Access-Control-Allow-Origin', '*');
		next();
	});
	
	const getScriptPubKeyFromAddress = (addr: string) => {
		try {
			return bitcoin.address.toOutputScript(addr, NETWORK);
		} catch(e) {
			if(config.debug) console.log(e);
			return false;
		}
	};
	
	// Get current block number (height).
	const getBlocks = async (): Promise<number> => {
		const status = await syncer.getStatus();
		return status.blocks;
	};
	
	// GraphQL.
	app.options(`${config.server.api.prefix}/graphql`, (req, res, next) => {
		res.header('Access-Control-Allow-Headers', 'Content-Type, Authorization, Content-Length, X-Requested-With');
		res.sendStatus(200);
	});
	app.use(`${config.server.api.prefix}/graphql`, express_graphql({
		schema: buildSchema(GQL_SCHEMA),
		rootValue: {
			status: async (): Promise<{blocks: number}> => ({
				blocks: await getBlocks(),
			}),
			block: async (args: {id: string}): Promise<cs.Block|null> => {
				try {
					if(args.id.match(/[0-9]+/)) {
						return await fetchBlockByHeight(syncer, Number.parseInt(args.id));
					} else {
						return await fetchBlockByHash(syncer, args.id);
					}
				} catch(e) {
					if(config.debug) console.log(e);
					return null;
				}
			},
			tx: async (args: {id: string}): Promise<cs.Transaction|null> => {
				try {
					return await fetchTransaction(syncer, args.id);
				} catch(e) {
					if(config.debug) console.log(e);
					return null;
				}
			},
			txids: async (args: {addr: string}): Promise<string[]|null> => {
				try {
					return await fetchTxids(args.addr);
				} catch(e) {
					if(config.debug) console.log(e);
					return null;
				}
			},
			utxos: async (args: {addr: string}): Promise<cs.Utxo[]|null> => {
				try {
					return await fetchUtxos(args.addr);
				} catch(e) {
					if(config.debug) console.log(e);
					return null;
				}
			},
			blockSummary: async (args: { offset: number, limit: number }) => {
				try {
					if(args.limit <= 0) throw new Error('the parameter "limit" should be greater than zero.');
					if(args.offset < 0) throw new Error('the parameter "offset" should be positive.');
					return await syncer.getBlockSummary(args.offset, args.limit);
				} catch(e) {
					if(config.debug) console.log(e);
				}
			},
			addressBalances: async(args: { offset: number, limit: number }) => {
				try {
					if(args.limit <= 0) throw new Error('the parameter "limit" should be greater than zero.');
					if(args.offset < 0) throw new Error('the parameter "offset" should be positive.');
					const count = await syncer.getRichListCount();
					if(args.offset >= count) throw new Error('the parameter "offset" exceeds the number of the records.');
					const entries = await syncer.getRichList(args.offset, args.limit);
					return {
						count: count,
						data: entries.map((entry) => ({
							scriptPubKey: entry.script_pubkey,
							address: resolveAddress(Buffer.from(entry.script_pubkey, 'hex'), NETWORK),
							value: entry.value.toString(),
						})),
					};
				} catch(e) {
					if(config.debug) console.log(e);
				}
			},
		},
		graphiql: true,
	}));
	
	// /status
	app.get(`${config.server.api.prefix}/${API_VERSION}/status`, async (req, res, next) => {
		res.json({
			blocks: await getBlocks(),
		});
	});
	
	// /block/[blockid]
	app.get(`${config.server.api.prefix}/${API_VERSION}/block/:blockid([0-9a-fA-F]{64})`, async (req, res, next) => {
		try {
			res.json(await fetchBlockByHash(syncer, req.params.blockid));
		} catch(e) {
			if(config.debug) console.log(e);
			res.status(404).json({ error: 'Block not found.' });
		}
	});
	
	// /block/[height]
	app.get(`${config.server.api.prefix}/${API_VERSION}/block/:height([0-9]+)`, async (req, res, next) => {
		try {
			res.json(await fetchBlockByHeight(syncer, Number.parseInt(req.params.height)));
		} catch(e) {
			if(config.debug) console.log(e);
			res.status(404).json({ error: 'Block not found.' })
		}
	});
	
	// /tx/[txid]
	app.get(`${config.server.api.prefix}/${API_VERSION}/tx/:txid([0-9a-fA-F]{64})`, async (req, res, next) => {
		try {
			res.json(await fetchTransaction(syncer, req.params.txid));
		} catch(e) {
			if(config.debug) console.log(e);
			res.status(404).json({ error: 'Transaction not found.' })
		}
	});
	
	// /txids/[address (single sig)]
	const fetchTxids = async (addr: string): Promise<string[]> => {
		const scriptPub = getScriptPubKeyFromAddress(addr);
		if(!scriptPub) throw new Error('Input address is invalid.');
		const txids = await syncer.getAddressIndex(scriptPub);
		return txids;
	};
	app.get(`${config.server.api.prefix}/${API_VERSION}/txids/:addr([0-9a-zA-Z]+)`, async (req, res, next) => {
		try {
			res.json(await fetchTxids(req.params.addr));
		} catch(e) {
			if(config.debug) console.log(e);
			res.status(404).json({ error: e.toString() });
		}
	});
	
	// /tx/broadcast
	app.put(`${config.server.api.prefix}/${API_VERSION}/tx/broadcast`, (req, res, next) => {
		if(typeof req.body != 'string') {
			res.status(400).json({ error: 'Request body is empty or has an invalid content-type (should be text/plain).' });
			return;
		}
		rpcClient.call('sendrawtransaction', [req.body], async (err, result) => {
			if(err) {
				res.status(400).json({ error: 'Failed to broadcast transaction.', message: err.toString() });
				return;
			}
			res.json(await fetchTransaction(syncer, result));
		});
	});
	
	// /utxos/{addr}
	const fetchUtxos = async (addr: string): Promise<cs.Utxo[]> => {
		const scriptPub = getScriptPubKeyFromAddress(addr);
		if(!scriptPub) throw new Error('Input address is invalid.');
		const syncerUtxos = await syncer.getUtxo(scriptPub);
		return syncerUtxos.map((utxo) => ({
			txid: utxo.txid,
			vout: utxo.vout,
			scriptPubKey: {
				asm: bitcoin.script.toASM(scriptPub),
				hex: scriptPub.toString('hex'),
				type: bitcoinClassify.output(scriptPub),
				address: resolveAddress(scriptPub, NETWORK),
			},
			value: utxo.value,
		}));
	};
	app.get(`${config.server.api.prefix}/${API_VERSION}/utxos/:addr([0-9a-zA-Z]+)`, async (req, res, next) => {
		try {
			res.json(await fetchUtxos(req.params.addr));
		} catch(e) {
			if(config.debug) console.log(e);
			res.status(404).json({ error: e.toString() });
		}
	});
	
	// Handle 404 errors.
	app.use((req, res, next) => {
		res.status(404);
		res.json({
			error: 'Requested endpoint not found.',
		});
	});
	
	// Handle 500 errors.
	app.use((err: Error, req: express.Request, res: express.Response, next: express.NextFunction) => {
		res.status(500);
		let json = {
			error: 'Internal error occured. Please contact administrator!',
		};
		if(config.debug) {
			(<any>json).message = err.toString();
			(<any>json).stack = err.stack;
		}
		res.json(json);
	});
	
	const server = app.listen(config.server.api.port, config.server.api.host, () => {
		console.log(`chainseeker-api is listening on ${config.server.api.host}:${config.server.api.port}`);
	});
	
	// Initialize WebSocketRelay.
	const wsr = new WebSocketRelay(config.coind.zmq, server);
	
};

main();

