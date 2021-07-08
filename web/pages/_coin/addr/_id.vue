<template>
	<div>
		<h1>Address <small>{{ address }}</small></h1>
		<div v-if="txids && utxos">
			<div class="mt-4">
				<v-row>
					<v-col md=2>Final Balance</v-col>
					<v-col md=4><Amount :value="utxos.reduce((acc, utxo) => acc + utxo.value, 0)" /></v-col>
					<v-col md=2>Rank</v-col>
					<v-col md=4 v-if="rank">#{{ rank.toLocaleString() }}</v-col>
					<v-col md=4 v-else style="color: dark-grey">unknown</v-col>
				</v-row>
				<v-row>
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
							<NuxtLink :to="`../tx/${item.txid}`">{{ item.txid }}</NuxtLink>
						</template>
					</v-data-table>
				</v-tab-item>
				<v-tab-item key="txs">
					<v-pagination total-visible=10 :value="txsPage + 1" :length="Math.ceil(txids.length / TXS_PER_PAGE)"
						v-on:input="(page) => { txsPage = page - 1; fetchTxs(); }" />
					<div v-for="tx in txs" :key="tx.txid" class="my-4">
						<v-row style="border-bottom: 1px solid gray; border-left: 5px solid #ccc;">
							<v-col><strong><NuxtLink :to="`../tx/${tx.txid}`">{{ tx.txid }}</NuxtLink></strong></v-col>
							<v-col v-if="tx.confirmedHeight >= 0" class="text-right">
								<small>
									Confirmed at <NuxtLink :to="`../block/${tx.confirmedHeight}`">#{{ tx.confirmedHeight.toLocaleString() }}</NuxtLink>
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
							<NuxtLink :to="`../tx/${item.txid}`">{{ item.txid }}</NuxtLink>
						</template>
						<template v-slot:item.value="{ item }">
							<Amount :value="item.value" />
						</template>
						<template v-slot:body.append>
							<tr>
								<th colspan="2" class="text-right">Total</th>
								<th><Amount :value="utxos.reduce((acc, utxo) => acc + utxo.value, 0)" /></th>
							</tr>
						</template>
					</v-data-table>
				</v-tab-item>
			</v-tabs-items>
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
	address: string | null = null;
	status: cs.Status | null = null;
	rank: number | null = null;
	txids: { txid: string }[] | null = null;
	txsPage: number = 0;
	txsBuffer: cs.Transaction[] = [];
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
	head() {
		return { title: `Address ${this.address} - chainseeker` };
	}
	async asyncData({ params, error, $config }: Context) {
		const address = params.id;
		const cs = new Chainseeker($config.coinConfig.apiEndpoint);
		const status = await cs.getStatus();
		try {
			const rank = await cs.getRichListRank(address);
			const txids = (await cs.getTxids(address)).map(txid => ({ txid: txid }));
			const utxos = await cs.getUtxos(address);
			return {
				rank,
				address,
				status,
				txids,
				txsPage: 0,
				txsBuffer: [],
				txs: [],
				utxos,
			};
		} catch(e) {
			error({ statusCode: 404, message: 'Address Not Found.' });
		}
	}
	async fetchTxs() {
		const cs = new Chainseeker(this.$config.coinConfig.apiEndpoint);
		this.txs = [];
		for(let i=this.txsPage*TXS_PER_PAGE; i<Math.min(this.txids!.length, (this.txsPage+1)*TXS_PER_PAGE); i++) {
			if(this.txsBuffer[i]) {
				this.txs.push(this.txsBuffer[i]);
			} else {
				this.txs.push(this.txsBuffer[i] = await cs.getTransaction(this.txids![i].txid));
			}
		}
	}
}
</script>

