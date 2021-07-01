<template>
	<div>
		<h1>Address <small>{{ address }}</small></h1>
		<div v-if="txids && utxos">
			<div class="mt-4">
				<v-row>
					<v-col md=2>Final Balance</v-col>
					<v-col md=4><Amount :value="utxos.reduce((acc, utxo) => acc + utxo.value, 0)" /></v-col>
					<v-col md=2>#transactions</v-col>
					<v-col md=4>{{ txids.length.toLocaleString() }}</v-col>
				</v-row>
			</div>
			<API :path="`txids/${address}`" />
			<v-tabs v-model="tab" grow>
				<v-tabs-slider></v-tabs-slider>
				<v-tab key="txids">
					Transaction IDs
				</v-tab>
				<v-tab key="txs" v-on:change="fetchTxs">
					Transactions
				</v-tab>
				<v-tab key="utxos">
					UTXOs
				</v-tab>
			</v-tabs>
			<v-tabs-items v-model="tab">
				<v-tab-item key="txids">
					<v-data-table :headers="txidHeaders" :items="txids">
						<template v-slot:item.txid="{ item }">
							<NuxtLink :to="`/tx/${item.txid}`">{{ item.txid }}</NuxtLink>
						</template>
					</v-data-table>
				</v-tab-item>
				<v-tab-item key="txs">
					<div v-for="tx in txs" class="my-4">
						<v-row style="border-bottom: 1px solid gray; border-left: 5px solid #ccc;">
							<v-col><strong><NuxtLink :to="`/tx/${tx.txid}`">{{ tx.txid }}</NuxtLink></strong></v-col>
							<v-col v-if="tx.confirmedHeight >= 0" class="text-right">
								<small>
									Confirmed at <NuxtLink :to="`/block/${tx.confirmedHeight}`">#{{ tx.confirmedHeight.toLocaleString() }}</NuxtLink>
								</small>
							</v-col>
							<v-col v-else class="text-right">
								<small>
									Unconfirmed
								</small>
							</v-col>
						</v-row>
						<v-row>
							<TxMovement :tx="tx" />
						</v-row>
					</div>
				</v-tab-item>
				<v-tab-item key="utxos">
					<v-data-table :headers="utxoHeaders" :items="utxos">
						<template v-slot:item.txid="{ item }">
							<NuxtLink :to="`/tx/${item.txid}`">{{ item.txid }}</NuxtLink>
						</template>
						<template v-slot:item.value="{ item }">
							<Amount :value="item.value" />
						</template>
					</v-data-table>
				</v-tab-item>
			</v-tabs-items>
		</div>
	</div>
</template>

<script lang="ts">
import { Vue, Component } from 'nuxt-property-decorator';
import { Chainseeker } from 'chainseeker';
import * as cs from 'chainseeker/dist/types';

@Component
export default class Home extends Vue {
	address: string | null = null;
	status: cs.Status;
	txids: { txid: string }[] | null = null;
	txs: cs.Transaction[] = [];
	utxos: cs.Utxo[]  = [];
	tab: string | null = null;
	data() {
		return {
			txidHeaders: [
				{ text: 'Transaction ID', value: 'txid' },
			],
			utxoHeaders: [
				{ text: 'Transaction ID', value: 'txid' },
				{ text: 'vout', value: 'vout' },
				{ text: 'Value', value: 'value' },
			],
		};
	}
	async asyncData({ params, error, $config }) {
		const address = params.id;
		const cs = new Chainseeker($config.apiEndpoint);
		const status = await cs.getStatus();
		try {
			const txids = (await cs.getTxids(address)).map(txid => ({ txid: txid }));
			const utxos = await cs.getUtxos(address);
			return {
				address,
				status,
				txids,
				utxos,
			};
		} catch(e) {
			error({ statusCode: 404, message: 'Address Not Found.' });
		}
	}
	async fetchTxs() {
		const cs = new Chainseeker(this.$config.apiEndpoint);
		this.txs = await cs.getTxs(this.address);
	}
}
</script>

