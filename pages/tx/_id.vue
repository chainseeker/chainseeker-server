<template>
	<div v-if="tx">
		<h1>
				Transaction
				<small>[{{ tx.txid.slice(0, 16) }}...]</small>
		</h1>
		<div class="text-center">
			<v-badge
				:color="confirmations === null ? 'red' : (confirmations >= 6 ? 'green' : 'yellow darken-3')"
				:content="confirmations === null ? 'unconfirmed' : confirmations + ' confirmations'">
			</v-badge>
		</div>
		<div class="my-4">
			<v-tooltip bottom>
				<template v-slot:activator="{ on, attrs }">
					<v-row class="my-2" v-bind="attrs" v-on="on">
						<v-col md=2><strong>Transaction ID</strong></v-col>
						<v-col md=10>{{ tx.txid }}</v-col>
					</v-row>
				</template>
				<span>The transaction ID (reverse of transaction hash).</span>
			</v-tooltip>
			<v-tooltip bottom>
				<template v-slot:activator="{ on, attrs }">
					<v-row class="my-2" v-bind="attrs" v-on="on">
						<v-col md=2><strong>Hash</strong></v-col>
						<v-col md=10>{{ tx.hash }}</v-col>
					</v-row>
				</template>
				<span>The hash of the transaction including witness (will coincides with txid if a transaction has no witness data).</span>
			</v-tooltip>
			<v-row class="my-2">
				<v-col md=2><strong>Size</strong></v-col>
				<v-col md=2>{{ tx.size.toLocaleString() }} bytes</v-col>
				<v-col md=2><strong>Virtual Size</strong></v-col>
				<v-col md=2>{{ tx.vsize.toLocaleString() }} bytes</v-col>
				<v-col md=2><strong>Weight</strong></v-col>
				<v-col md=2>{{ tx.weight.toLocaleString() }} WU</v-col>
			</v-row>
			<v-row class="my-2">
				<v-col md=2><strong>Version</strong></v-col>
				<v-col md=4>{{ tx.version.toLocaleString() }}</v-col>
				<v-col md=2><strong>Lock Time</strong></v-col>
				<v-col md=4>{{ tx.locktime.toLocaleString() }}</v-col>
			</v-row>
			<v-row class="my-2">
				<v-col md=2><strong>Confirmed Height</strong></v-col>
				<v-col md=4>
					<span v-if="tx.confirmedHeight">
						<NuxtLink :to="`/block/${tx.confirmedHeight}`">
							{{ tx.confirmedHeight.toLocaleString() }}
						</NuxtLink>
						<span class="ml-4">({{ new Date(1000 * blockHeader.time).toLocaleString() }})</span>
					</span>
					<span v-else>
						{{ tx.confirmedHeight ? tx.confirmedHeight.toLocaleString() : 'unconfirmed' }}
					</span>
				</v-col>
				<v-col md=2><strong>Fee</strong></v-col>
				<v-col md=4>
					<Amount :value="tx.fee" />
					(<Amount :value="Math.floor(tx.fee / tx.size)" :symbol="`${$config.coin.satoshi} / byte`" :unitInSatoshi="true" />)
				</v-col>
			</v-row>
		</div>
		<h2>Transaction Details</h2>
		<div class="my-4">
			<v-row>
				<v-col class="text-center"><strong>Input</strong></v-col>
				<v-col class="text-center"><strong>Output</strong></v-col>
			</v-row>
			<TxMovement :tx="tx" />
		</div>
		<API :path="`tx/${tx.txid}`" />
	</div>
</template>

<script lang="ts">
import { Vue, Component } from 'nuxt-property-decorator';
import { Chainseeker } from 'chainseeker';
import * as cs from 'chainseeker/dist/types';

@Component
export default class Home extends Vue {
	status: cs.Status;
	status?: cs.Status | null = null;
	tx?: cs.Transaction | null = null;
	blockHeader: cs.BlockHeader | null = null;
	confirmations: number | null = null;
	async mounted() {
		const cs = new Chainseeker(this.$config.apiEndpoint);
		this.status = await cs.getStatus();
		const tx = await cs.getTransaction(this.$route.params.id);
		if(tx.confirmedHeight) {
			this.blockHeader = await cs.getBlockHeader(tx.confirmedHeight);
		}
		this.tx = tx;
		this.confirmations = tx.confirmedHeight ? this.status.blocks - tx.confirmedHeight + 1 : null;
	}
}
</script>

