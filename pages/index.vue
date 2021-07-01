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
						<td>{{ block.ntxs }}</td>
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
						<th>Received at</th>
						<th>Transaction ID</th>
						<th># of vins</th>
						<th># of vouts</th>
						<th>Size</th>
						<th>Value Transacted</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="{ received, tx } in recentTxs">
						<td><ElapsedTime :time="received" /> ago</td>
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

const MAX_LATEST_BLOCKS = 5;
const MAX_LATEST_TXS = 5;

@Component
export default class Home extends Vue {
	recentBlocks: cs.BlockHeader[] = [];
	recentTxs: {
		received: number,
		tx: cs.Transaction,
	}[] = [];
	initWebSocket() {
		const cs = new Chainseeker(this.$config.apiEndpoint);
		const ws = new WebSocket(this.$config.wsEndpoint);
		ws.onmessage = async (msg) => {
			const data = JSON.parse(msg.data);
			switch(data[0]) {
				case 'hashtx':
					this.recentTxs.unshift({ received: Date.now(), tx: await cs.getTransaction(data[1]) });
					if(this.recentTxs.length > MAX_LATEST_TXS) {
						this.recentTxs.splice(0, this.recentTxs.length - MAX_LATEST_TXS);
					}
					break;
				case 'hashblock':
					this.recentBlocks.unshift(await cs.getBlockHeader(data[1]));
					this.recentBlocks.pop();
					break;
				default:
			}
		};
	}
	async asyncData({ params, error, $config }) {
		const cs = new Chainseeker($config.apiEndpoint);
		// Fetch status.
		const status = await cs.getStatus();
		// Fetch recent blocks.
		const recentBlocks = [];
		for(let height=status.blocks; height>=status.blocks-MAX_LATEST_BLOCKS; height--) {
			recentBlocks.push(await cs.getBlockHeader(height));
		}
		return {
			status,
			recentBlocks,
		};
	}
	mounted() {
		// Initialize WebSocket connection.
		this.initWebSocket();
	}
}
</script>

