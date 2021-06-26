
import { exec } from 'child_process';

import express from 'express';
import morgan from 'morgan';
import compression from 'compression';

import { Chainseeker } from 'chainseeker';
import * as csTypes from 'chainseeker/dist/types';

const main = async () => {
	const COIN_NAME = process.argv[2];
	const configs = (await import(`${process.env['HOME']}/.chainseeker/config.ts`)).configs;
	const config = configs[COIN_NAME];
	const cs = new Chainseeker(config.endpoint.apiLocal);
	
	process.on('unhandledRejection', console.dir);
	
	const app: express.Application = express();
	app.set('views', './web/views');
	app.set('view engine', 'ejs');
	app.use(`${config.server.web.prefix}/static`, express.static(__dirname + '/../static'));
	app.use(morgan('dev'));
	app.use(compression({
		filter: (req, res) => {
			return true;
		},
	}));
	
	app.locals.config = config;
	app.locals.formatAddress = (addr: string, colorize: string[] = []) => {
		if(!addr) return 'Unknown address';
		const color: string = (colorize.indexOf(addr) >= 0) ? ' style="color:red;"' : '';
		return [
			`<canvas width="30" height="30" style="margin:-8px 3px;" data-jdenticon-value="${addr}"></canvas>`,
			`<a href="${config.server.web.prefix}/addr/${addr}"${color}>${addr}</a>`,
		].join('');
	};
	app.locals.formatAmount = (amount: number, symbol: string=config.coin.symbol) => {
		return (amount * 1e-8).toFixed(8) + ' <small>' + symbol + '</small>';
	}
	
	class NotFoundError extends Error {}
	
	type Transaction = csTypes.Transaction & {
		block: Block;
	};
	
	type Block = csTypes.Block & {
		txs: Transaction[];
		reward: number;
		fee: number;
	};
	
	// Pre-hook.
	app.use(async (req, res, next) => {
		app.locals.processStartTime = new Date().getTime();
		app.locals.status = await cs.getStatus();
		next();
	});
	
	const SITEMAP_BLOCKS = 10;
	const SITEMAP_RICHLISTS = 10000;
	
	// robots.txt
	app.get(`${config.server.web.prefix}/robots.txt`, async (req, res, next) => {
		res.header('Content-Type', 'text/plain;charset=utf-8');
		const ret: string[] = [];
		ret.push(`Sitemap: ${config.endpoint.webPublic}/sitemap-static.txt`);
		for(let i=0; i*SITEMAP_BLOCKS<app.locals.status.blocks; i++) {
			ret.push(`Sitemap: ${config.endpoint.webPublic}/sitemap-${i}.txt`);
		}
		const addressBalances = await cs.getAddressBalances(0, 1);
		for(let i=0; i*SITEMAP_RICHLISTS<addressBalances.count; i++) {
			ret.push(`Sitemap: ${config.endpoint.webPublic}/sitemap-richlist-${i}.txt`);
		}
		res.send(ret.join('\n'));
	});
	
	// Sitemap (static pages)
	app.get(`${config.server.web.prefix}/sitemap-static.txt`, async (req, res, next) => {
		res.header('Content-Type', 'text/plain;charset=utf-8');
		res.send([
			'/',
			'/stat/nonce',
			'/stat/size',
		].map((x) => config.endpoint.webPublic+x).join('\n'));
	});
	
	// Sitemap (blocks, txs)
	app.get(`${config.server.web.prefix}/sitemap-:index([0-9]*).txt`, async (req, res, next) => {
		const index = Number.parseInt(req.params.index);
		const ret: string[] = [];
		for(let i=SITEMAP_BLOCKS*index; i<Math.min(SITEMAP_BLOCKS*(index+1), app.locals.status.blocks); i++) {
			const block = await cs.getBlock(i);
			ret.push(`${config.endpoint.webPublic}/block/${i}`);
			ret.push(`${config.endpoint.webPublic}/block/${block.hash}`);
			for(const txid of block.txids) {
				ret.push(`${config.endpoint.webPublic}/tx/${txid}`);
			}
		}
		if(ret.length == 0) {
			next(new NotFoundError('Invalid sitemap index.'));
			return;
		}
		res.header('Content-Type', 'text/plain;charset=utf-8');
		res.send(ret.join('\n'));
	});
	
	// Sitemap (richlist)
	app.get(`${config.server.web.prefix}/sitemap-richlist-:page([0-9]*).txt`, async (req, res, next) => {
		const page = Number.parseInt(req.params.page);
		const addressBalances = await cs.getAddressBalances(0, 1);
		const ret: string[] = [];
		if(page == 0) {
			ret.push(`${config.endpoint.webPublic}/stat/richlist`);
		}
		for(let i=page*SITEMAP_RICHLISTS+1; i<=Math.min((page+1)*SITEMAP_RICHLISTS, addressBalances.count); i++) {
			ret.push(`${config.endpoint.webPublic}/stat/richlist/${i}`);
		}
		if(ret.length == 0) {
			next(new NotFoundError('Invalid sitemap index.'));
			return;
		}
		res.header('Content-Type', 'text/plain;charset=utf-8');
		res.send(ret.join('\n'));
	});
	
	// top (/)
	app.get(`${config.server.web.prefix}/`, async (req, res, next) => {
		res.render('index.ejs');
	});
	
	// transaction (/tx)
	app.get(`${config.server.web.prefix}/tx/:txid([0-9a-fA-F]{64})`, async (req, res, next) => {
		try {
			const tx = <Transaction>await cs.getTransaction(req.params.txid);
			// Fetch block information.
			if(tx.confirmed_height) {
				tx.block = <Block>await cs.getBlock(tx.confirmed_height);
			}
			res.render('transaction.ejs', { tx: tx });
		} catch(e) {
			if(config.debug) console.log(e);
			next(new NotFoundError('Transaction not found.'));
		}
	});
	
	// block (/block)
	app.get(`${config.server.web.prefix}/block/:blocktag([0-9]+|[0-9a-fA-F]{64})`, async (req, res, next) => {
		try {
			// Fetch block from backend.
			const block = <Block>await cs.getBlock(req.params.blocktag);
			block.txs = <Transaction[]>await cs.getTransactions(block.txids);
			// Compute fee and block reward.
			// TODO: compute reward for re-orged blocks.
			block.reward = block.height ? config.coin.block_reward.initial * Math.pow(2, -Math.floor(block.height / config.coin.block_reward.halving)) : 0;
			block.fee = -(block.txs[0].fee + block.reward);
			res.render('block.ejs', { block: block });
		} catch(e) {
			if(config.debug) console.log(e);
			next(new NotFoundError('Block not found.'));
		}
	});
	
	// address (/addr)
	app.get(`${config.server.web.prefix}/addr/:addr([0-9a-zA-Z]+)`, async (req, res, next) => {
		let ret = {
			addr: req.params.addr,
			balance: 0,
			limit: 10,
			page: (req.query.page ? +req.query.page : 0),
			pageTotal: 0,
			order: (req.query.order=='ASC' ? 1 : -1),
			txids: <string[]>[],
			utxos: <csTypes.Utxo[]>[],
			txs: <Transaction[]>[],
		};
		if(ret.page < 0) ret.page=0;
		try {
			ret.txids = await cs.getTxids(req.params.addr);
			ret.pageTotal = Math.ceil((ret.txids.length-1) / ret.limit);
			if(ret.order == -1) {
				ret.txids.reverse();
			}
			ret.utxos = await cs.getUtxos(req.params.addr);
			ret.utxos.map((utxo) => { ret.balance += utxo.value; });
			// Get transactions.
			let getTxids: string[] = [];
			for(let i=ret.page*ret.limit; i<Math.min((ret.page+1)*ret.limit, (ret.txids).length); i++) {
				getTxids.push(ret.txids[i]);
			}
			ret.txs = <Transaction[]>await cs.getTransactions(getTxids);
			res.render('address.ejs', { addr: ret });
		} catch(e) {
			if(config.debug) console.log(e);
			next(new NotFoundError('Address has an invalid form.'));
		}
	});
	
	// /search (block id / txid?)
	app.get(`${config.server.web.prefix}/search/:id([0-9a-fA-F]{64})`, async (req, res, next) => {
		// Checks if transaction exists.
		try {
			await cs.getTransaction(req.params.id);
			res.redirect(301, `${config.server.web.prefix}/tx/${req.params.id}`);
		} catch(e) {
			// Checks if block exists.
			try {
				await cs.getBlock(req.params.id);
				res.redirect(301, `${config.server.web.prefix}/block/${req.params.id}`);
			} catch(e) {
				next(new NotFoundError('No block or transaction matching this hash found!'));
			}
		}
	});
	// /search (block height?)
	app.get(`${config.server.web.prefix}/search/:height([0-9]+)`, (req, res, next) => {
		if(req.params.height <= app.locals.status.blocks) {
			res.redirect(301, `${config.server.web.prefix}/block/${req.params.height}`);
		} else {
			next();
		}
	});
	// /search (address?)
	app.get(`${config.server.web.prefix}/search/:addr([0-9a-zA-Z]+)`, async (req, res, next) => {
		try {
			await cs.getTxids(req.params.addr);
			res.redirect(301, `${config.server.web.prefix}/addr/${req.params.addr}`);
		} catch(e) {
			next();
		}
	});
	
	// Stat >> Nonce Distribution (/stat/nonce)
	/*
	app.get(`${config.server.web.prefix}/stat/nonce`, async (req, res, next) => {
		res.render('stat/nonce.ejs');
	});
	app.get(`${config.server.web.prefix}/img/nonce.png`, async (req, res, next) => {
		if(noncePng) {
			res.header('Content-Type', 'image/png');
			res.send(noncePng);
		} else {
			next(new Error('currently rendering PNG image.'));
		}
	});
	*/
	
	// Stat >> Block Size (/stat/size)
	app.get(`${config.server.web.prefix}/stat/size`, async (req, res, next) => {
		res.render('stat/size.ejs');
	});
	
	// Stat >> Rich List (/stat/richlist)
	app.get(`${config.server.web.prefix}/stat/richlist/:page([0-9]+)?`, async (req, res, next) => {
		const page = Number.parseInt(req.params.page || '1');
		if(page <= 0) {
			next(new NotFoundError('page should be greater than or equals to 1.'));
			return;
		}
		const LIMIT = 100;
		try {
			const addressBalances = await cs.getAddressBalances((page-1)*LIMIT, LIMIT);
			res.render('stat/richlist.ejs', { LIMIT: LIMIT, page: page, addressBalances: addressBalances });
		} catch(e) {
			next(new NotFoundError('page is in an invalid range.'));
		}
	});
	
	// Handle 404 errors.
	app.use((req, res, next) => {
		res.status(404);
		res.render('errors/404.ejs', { message: 'The requested URL did not match with any endpoints.' });
	});
	
	// Handle 500 errors.
	app.use((err: Error, req: express.Request, res: express.Response, next: express.NextFunction) => {
		if(err instanceof NotFoundError) {
			res.status(404);
			res.render('errors/404.ejs', { message: err.message });
		} else {
			res.status(500);
			res.render('errors/500.ejs', { err: err });
		}
	});
	
	app.listen(config.server.web.port, config.server.web.host, () => {
		console.log(`chainseeker-web is listening on ${config.server.web.host}:${config.server.web.port}`);
	});
	
	// Plot nonce.
	/*
	let noncePng: Buffer|null = null;
	const plotNonce = () => {
		console.log('[plotNonce] Plotting nonce distribution...');
		exec(`${__dirname}/../plot-nonce.sh ${process.argv[2]}`, { encoding: 'buffer', maxBuffer: 100*1024*1024 }, (error, stdout, stderr) => {
			console.log(stderr.toString());
			noncePng = stdout;
			console.log('[plotNonce] Done.');
		});
	};
	setInterval(plotNonce, 60*60*1000);
	plotNonce();
	*/
};

main();

