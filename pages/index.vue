<template>
	<div>
		<h1>Recent Blocks</h1>
		<v-simple-table>
			<template v-slot:default>
				<thead>
					<tr>
						<th>Height</th>
						<th>Time</th>
						<th># of txs</th>
						<th>Size</th>
						<th>ID</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="block in recentBlocks">
						<td><NuxtLink :to="`/block/${block.height}`">{{ block.height }}</NuxtLink></td>
						<td><ElapsedTime :time="1000 * block.time" /> ago</td>
						<td>{{ block.txids.length }}</td>
						<td>{{ block.size.toLocaleString() }} bytes</td>
						<td>{{ block.hash }} bytes</td>
					</tr>
				</tbody>
			</template>
		</v-simple-table>
		<h1>Recent Transactions</h1>
		<v-simple-table>
			<template v-slot:default>
				<thead>
					<tr>
						<th>Transaction ID</th>
						<th># of vins</th>
						<th># of vouts</th>
						<th>Size</th>
						<th>Value Transacted</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="tx in recentTxs">
						<td><NuxtLink :to="`/tx/${tx.txid}`">{{ tx.txid }}</NuxtLink></td>
						<td>{{ tx.vin.length.toLocaleString() }}</td>
						<td>{{ tx.vout.length.toLocaleString() }}</td>
						<td>{{ tx.size.toLocaleString() }} bytes</td>
						<td><Amount :value="tx.vout.reduce((acc, vout) => (acc + vout.value), 0)" /></td>
					</tr>
				</tbody>
			</template>
		</v-simple-table>
	</div>
</template>

<script lang="ts">
import { Vue, Component } from 'nuxt-property-decorator';
import { Chainseeker } from 'chainseeker';
import * as cs from 'chainseeker/dist/types';

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

@Component
export default class Home extends Vue {
	const MAX_LATEST_BLOCKS = 5;
	const MAX_LATEST_TXS = 5;
	cs: Chainseeker;
	recentBlocks: cs.Blocks[] = [];
	recentTxs: cs.Transaction[] = [];
	constructor() {
		super();
		this.cs = new Chainseeker(this.$config.apiEndpoint);
	}
	initWebSocket() {
		const ws = new WebSocket(this.$config.wsEndpoint);
		ws.onmessage = async (msg) => {
			const data = JSON.parse(msg.data);
			switch(data[0]) {
				case 'hashtx':
					this.recentTxs.unshift(await this.cs.getTransaction(data[1]));
					if(this.recentTxs.length > this.MAX_LATEST_TXS) {
						this.recentTxs.splice(0, this.recentTxs.length - this.MAX_LATEST_TXS);
					}
					break;
				case 'hashblock':
					this.recentBlocks.unshift(await this.cs.getBlock(data[1]));
					this.recentBlocks.pop();
					break;
				default:
			}
		};
	}
	async mounted() {
		// Fetch status.
		const status = await this.cs.getStatus();
		// Fetch recent blocks.
		for(let height=status.blocks; height>=status.blocks-MAX_LATEST_BLOCKS; height--) {
			this.recentBlocks.push(await this.cs.getBlock(height));
		}
		// Initialize WebSocket connection.
		this.initWebSocket();
	}
}
</script>

