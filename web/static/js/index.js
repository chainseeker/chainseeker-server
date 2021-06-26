
chainseeker.setApiEndPoint(public_endpoint);

function formatAmount(amount, s=symbol) {
	return (amount * 1e-8).toFixed(8) + ' <small>' + s + '</small>';
}

function formatElapsedTime(time) {
	const abs = Math.abs(time);
	// 0 ~ 1sec
	if(abs < 1000) {
		return `${time} ms`;
	}
	// 1sec ~ 1min
	if(abs < 60 * 1000) {
		return `${(time/1000.).toFixed(2)} sec`;
	}
	// 1min ~ 1hour
	if(abs < 60 * 60 * 1000) {
		return `${(time/60./1000.).toFixed(0)} min ${((time%(60*1000))/1000.).toFixed(0)} sec`;
	}
	// 1hour ~ 1day
	if(abs < 24 * 60 * 60 * 1000) {
		return `${(time/60./60./1000.).toFixed(0)} hour ${((time%(60*60*1000))/60./1000.).toFixed(0)} min`;
	}
	// 1day ~
	return `${(time/24./60./60./1000.).toFixed(0)} day ${((time%(24*60*60*1000))/60./60./1000.).toFixed(0)} hour`;
}

/****************************************************************************************************
 * Latest Transactions.
 ****************************************************************************************************/

const MAX_LATEST_TXS = 10;
let txs = [];
let lastTxCount = 0;

function updateLatestTxTime() {
	if(txs.length == 0) return;
	for(let i=txs.length-1; i>=Math.max(0, txs.length-MAX_LATEST_TXS); i--) {
		const idx = txs.length - i;
		const html = formatElapsedTime(new Date().getTime() - (txs[i].time)) + ' ago';
		$('#latest-txs tbody tr:nth-child('+idx+') td:nth-child(2)').html(html);
	}
}

function updateLatestTxs() {
	if(txs.length == 0) return;
	if(txs.length == lastTxCount) {
		updateLatestTxTime();
		return;
	}
	lastTxCount = txs.length;
	let html = '';
	for(let i=txs.length-1; i>=Math.max(0, txs.length-MAX_LATEST_TXS); i--) {
		const tx = txs[i].tx;
		html += [
			'<tr>',
				'<td><a href="' + server_prefix + '/tx/' + tx.txid + '">' + tx.txid + '</a></td>',
				'<td>' + formatElapsedTime(new Date().getTime() - txs[i].time) + ' sec ago</td>',
				'<td>' + tx.size.toLocaleString() + ' bytes</td>',
				'<td>' + formatAmount(tx.vout.reduce((a, b) => (a + b.value), 0)) + '</td>',
			'</tr>',
		].join('');
	}
	$('#latest-txs tbody').html(html);
}

function insertTx(txid) {
	return chainseeker.getTransaction(txid).then((tx) => {
		if(!tx) return;
		txs.push({
			tx: tx,
			time: new Date().getTime(),
		});
	});
}

/****************************************************************************************************
 * Latest Blocks.
 ****************************************************************************************************/

const MAX_LATEST_BLOCKS = 5;
let blocks = [];
let lastBlockCount = 0;

function updateLatestBlockTime() {
	if(blocks.length == 0) return;
	for(let i=blocks.length-1; i>=Math.max(0, blocks.length-MAX_LATEST_BLOCKS); i--) {
		const idx = blocks.length - i;
		const html = formatElapsedTime(new Date().getTime() - 1000 * (blocks[i].time)) + ' ago';
		$('#latest-blocks tbody tr:nth-child('+idx+') td:nth-child(2)').html(html);
	}
}

function updateLatestBlocks() {
	if(blocks.length == 0) return;
	if(blocks.length == lastBlockCount) {
		updateLatestBlockTime();
		return;
	}
	lastBlockCount = blocks.length;
	let html = '';
	for(let i=blocks.length-1; i>=Math.max(0, blocks.length-MAX_LATEST_BLOCKS); i--) {
		const block = blocks[i];
		html += [
			'<tr>',
				'<td><a href="' + server_prefix + '/block/' + block.height + '">',
					block.height.toLocaleString(),
				'</a></td>',
				'<td>' + formatElapsedTime(new Date().getTime() - 1000 * (block.time)) + ' ago</td>',
				'<td>' + block.txids.length.toLocaleString() + '</td>',
				'<td>' + block.size.toLocaleString() + ' bytes</td>',
				'<td style="font-family:monospace;">' + block.hash + '</td>',
			'</tr>',
		].join('');
	}
	$('#latest-blocks tbody').html(html);
}

function insertBlock(blockid) {
	return chainseeker.getBlock(blockid).then((block) => {
		if(!block) return;
		blocks.push(block);
	});
}

/****************************************************************************************************
 * Initialize.
 ****************************************************************************************************/

$(function() {
	// Load latest blocks.
	let refs = [];
	for(let i=MAX_LATEST_BLOCKS-1; i>=0; i--) {
		refs.push(bestheight - i);
	}
	chainseeker.getBlocks(refs).then((bs) => {
		blocks = bs;
	});
	setInterval(updateLatestTxs, 500);
	setInterval(updateLatestBlocks, 500);
	// Initialize WebSocket connection.
	ws = new WebSocket(websocket_endpoint, 'chainseeker');
	ws.onmessage = function(msg) {
		const data = JSON.parse(msg.data);
		switch(data[0]) {
			case 'hashtx':
				insertTx(data[1]);
				break;
			case 'hashblock':
				insertBlock(data[1]);
				break;
			default:
		}
	};
});

