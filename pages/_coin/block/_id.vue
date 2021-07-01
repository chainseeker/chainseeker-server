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
						<td colspan="3">{{ block.txs.length.toLocaleString() }}</td>
					</tr>
					<tr>
						<th>Block Reward</th>
						<td><Amount :value="block.txs[0].vout[0].value" /></td>
						<th>Transaction Fee in Total</th>
						<td><Amount :value="fee" /></td>
					</tr>
				</tbody>
			</template>
		</v-simple-table>
		<API :path="`block/${block.hash}`" />
		<h2>Transactions in the Block</h2>
		<div v-for="(tx, n) in block.txs" class="my-4">
			<v-row style="border-bottom: 1px solid gray; border-left: 5px solid #ccc;">
				<v-col><strong><NuxtLink :to="`../tx/${tx.txid}`">{{ tx.txid }}</NuxtLink></strong></v-col>
				<v-col v-if="n !== 0" class="text-right">(fee: <Amount :value="tx.fee" />)</v-col>
				<v-col v-else         class="text-right">(reward: <Amount :value="-tx.fee" />)</v-col>
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

@Component
export default class Home extends Vue {
	status: cs.Status | null = null;
	block?: cs.BlockWithTxs | null = null;
	fee: number = 0;
	head() {
		return { title: `Block ${this.block!.hash!} - chainseeker` };
	}
	async asyncData({ params, error, $config }: Context) {
		const cs = new Chainseeker($config.coinConfig.apiEndpoint);
		const status = await cs.getStatus();
		// Fetch block.
		try {
			const block = await cs.getBlockWithTxs(params.id);
			// Compute fee.
			let fee = 0;
			for(let n=1; n<block.txs.length; n++) {
				let tx = block.txs[n];
				fee += tx.vin.reduce((acc, vin) => acc + vin.value, 0);
				fee -= tx.vout.reduce((acc, vout) => acc + vout.value, 0);
			}
			return {
				status,
				block,
				fee,
			};
		} catch(e) {
			error({ statusCode: 404, message: 'Block Not Found.' });
		}
	}
}
</script>

