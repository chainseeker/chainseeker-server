<template>
	<div v-if="block">
		<h1>Block <small>#{{ block.height.toLocaleString() }}</small></h1>
		<div class="text-center">
			<v-pagination total-visible=10 :value="block.height" :length="status.blocks"
				v-on:input="(height) => $router.push('/block/' + height)" />
		</div>
		<v-simple-table>
			<template v-slot:default>
				<tbody>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<tr v-bind="attrs" v-on="on">
								<th>Block ID</th>
								<td colspan="3">{{ block.hash }}</td>
							</tr>
						</template>
						<span>The hash of the block header (reversed).</span>
					</v-tooltip>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<tr v-bind="attrs" v-on="on">
								<th>Previous Block ID</th>
								<td colspan="3"><NuxtLink :to="`./${block.previousblockhash}`">{{ block.previousblockhash }}</NuxtLink></td>
							</tr>
						</template>
						<span>The hash of the previous block mined.</span>
					</v-tooltip>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<tr v-bind="attrs" v-on="on">
								<th>Merkle Root</th>
								<td colspan="3">{{ block.merkleroot }}</td>
							</tr>
						</template>
						<span>Merkle root of the transactions included in this block.</span>
					</v-tooltip>
					<tr>
						<th>Time</th>
						<td>{{ new Date(1000 * block.time).toLocaleString() }}</td>
						<th>Version</th>
						<td>{{ block.version }} (0x{{ block.version.toString(16).padStart(8, '0') }})</td>
					</tr>
					<tr>
						<th>Bits</th>
						<td>{{ block.bits }}</td>
						<th>Difficulty</th>
						<td>{{ block.difficulty }}</td>
					</tr>
					<tr>
						<th>Nonce</th>
						<td colspan="3">{{ block.nonce }} (0x{{ block.nonce.toString(16).padStart(8, '0') }})</td>
					</tr>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<tr v-bind="attrs" v-on="on">
								<th>Size</th>
								<td colspan="3">{{ block.size.toLocaleString() }} bytes</td>
							</tr>
						</template>
						<span>The size of the block (includes witness data).</span>
					</v-tooltip>
					<tr>
						<th>Stripped Size</th>
						<td>{{ block.strippedsize.toLocaleString() }} bytes</td>
						<th>Weight</th>
						<td>{{ block.weight.toLocaleString() }} WU</td>
					</tr>
					<tr>
						<th>#transactions</th>
						<td colspan="3">{{ block.txids.length.toLocaleString() }}</td>
					</tr>
					<tr>
						<th>Block Reward</th>
						<td><Amount :value="coinbase.vout[0].value" /></td>
						<th>Generated Coins</th>
						<td><Amount :value="generatedAmount" /></td>
					</tr>
					<tr>
						<th>Transaction Fee in Total</th>
						<td><Amount :value="fee" /></td>
					</tr>
				</tbody>
			</template>
		</v-simple-table>
		<API :path="`block/${block.hash}`" />
		<h2>Transactions in this Block</h2>
		<div class="text-center">
			<v-pagination total-visible=10 :value="page + 1" :length="Math.ceil(block.txids.length / TXS_PER_PAGE)"
				v-on:input="(page) => $router.push(`/${coin}/block/${block.height}/${page - 1}`)" />
		</div>
		<div v-for="(tx, n) in txs" class="my-4">
			<v-row style="border-bottom: 1px solid gray; border-left: 5px solid #ccc;">
				<v-col><strong><NuxtLink :to="`../tx/${tx.txid}`">{{ tx.txid }}</NuxtLink></strong></v-col>
				<v-col v-if="page == 0 && n == 0" class="text-right">(reward: <Amount :value="-tx.fee" />)</v-col>
				<v-col v-else                     class="text-right">(fee: <Amount :value="tx.fee" />)</v-col>
			</v-row>
			<v-row>
				<TxMovement :tx="tx" />
			</v-row>
		</div>
	</div>
</template>

<script lang="ts">
import { Context } from '@nuxt/types';
import { Vue, Component } from 'nuxt-property-decorator';
import { Chainseeker } from 'chainseeker';
import * as cs from 'chainseeker/dist/types';

const TXS_PER_PAGE = 10;

@Component
export default class Home extends Vue {
	TXS_PER_PAGE: number = TXS_PER_PAGE;
	coin?: string;
	page?: number;
	status?: cs.Status;
	block?: cs.BlockWithTxids;
	coinbase?: cs.Transaction;
	txs: cs.Transaction[] = [];
	generatedAmount?: number;
	fee: number = 0;
	head() {
		return { title: `Block ${this.block!.hash!} - chainseeker` };
	}
	async asyncData({ params, error, $config }: Context) {
		const coin = params.coin;
		const page = params.page ? Number.parseInt(params.page) : 0;
		const cs = new Chainseeker($config.coinConfig.apiEndpoint);
		const status = await cs.getStatus();
		// Fetch block.
		try {
			const block = await cs.getBlockWithTxids(params.id);
			// Fetch transactions.
			const txPromises: Promise<cs.Transaction>[] = [];
			for(let i=page*TXS_PER_PAGE; i<Math.min(block.txids.length, (page+1)*TXS_PER_PAGE); i++) {
				txPromises.push(cs.getTransaction(block.txids[i]));
			}
			const txs = await Promise.all(txPromises);
			// Fetch coinbase transaction.
			const coinbase = await (async () => {
				if(page == 0) {
					return txs[0];
				} else {
					return await cs.getTransaction(block.txids[0]);
				}
			})();
			// Compute the amount of newly generated coins.
			const blockReward = $config.coinConfig.coin.blockReward;
			const generatedAmount = blockReward.initial * Math.pow(0.5, Math.floor(block.height / blockReward.halving));
			// Compute fee.
			const fee = coinbase.vout[0].value - generatedAmount;
			return {
				coin,
				page,
				status,
				block,
				coinbase,
				txs,
				generatedAmount,
				fee,
			};
		} catch(e) {
			console.log(e);
			error({ statusCode: 404, message: 'Block Not Found.' });
		}
	}
}
</script>

