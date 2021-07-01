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
								<td colspan="3"><NuxtLink :to="`/block/${block.previousblockhash}`">{{ block.previousblockhash }}</NuxtLink></td>
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
						<td><Amount :value="block.reward" /></td>
						<th>Transaction Fee in Total</th>
						<td><Amount :value="block.fee" /></td>
					</tr>
				</tbody>
			</template>
		</v-simple-table>
		<API :path="`block/${block.hash}`" />
		<h2>Transactions in the Block</h2>
		<div v-for="tx in block.txs" class="my-4">
			<v-container>
				<v-row style="border-bottom: 1px solid gray; border-left: 5px solid #ccc;">
					<v-col><strong><NuxtLink to="/tx/${tx.txid}">{{ tx.txid }}</NuxtLink></strong></v-col>
					<v-col v-if="tx.address !== 'coinbase'" class="text-right">(fee: <Amount :value="tx.fee" />)</v-col>
					<v-col v-else                           class="text-right">(reward: <Amount :value="tx.fee" />)</v-col>
				</v-row>
				<v-row>
					<TxMovement :tx="tx" />
				</v-row>
			</v-container>
		</div>
	</div>
</template>

<script lang="ts">
import { Vue, Component } from 'nuxt-property-decorator';
import { Chainseeker } from 'chainseeker';
import * as cs from 'chainseeker/dist/types';

@Component
export default class Home extends Vue {
	status: cs.Status;
	block?: cs.BlockWithTxs | null = null;
	constructor() {
		super();
	}
	async mounted() {
		const cs = new Chainseeker(this.$config.apiEndpoint);
		this.status = await cs.getStatus();
		// Fetch block.
		const block = await cs.getBlockWithTxs(this.$route.params.id);
		this.block = block;
	}
}
</script>

