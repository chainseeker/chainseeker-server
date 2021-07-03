<template>
	<div>
		<h1>chainseeker - an open-source blockchain explorer.</h1>
		<v-row class="mt-8">
			<v-col md=3 v-for="(coinConfig, coin) in $config.coins" v-if="coin != 'local'" :key="coin">
				<v-card>
					<v-card-title>{{ coinConfig.coin.name }}</v-card-title>
					<v-card-text>
						<div>
							<NuxtLink :to="coin">
								<img :src="status[coin].icon" :alt="`${coinConfig.coin.name} icon`" width="100%" />
							</NuxtLink>
						</div>
						<div class="mt-4">
							<template v-if="status[coin].blocks">
								<p>Synced height: {{ status[coin].blocks.toLocaleString() }}</p>
								<p>
									<v-badge :color="Date.now() - 1000 * status[coin].lastBlock.time < 60 * 60 * 1000 ? 'green' : 'red'" inline>
										Last block: {{ new Date(1000 * status[coin].lastBlock.time).toLocaleString() }}
									</v-badge>
								</p>
							</template>
							<template v-else>
								<v-alert type="error">
									Server is down.
								</v-alert>
								<p>If the error continues, please contact the admin.</p>
							</template>
						</div>
					</v-card-text>
				</v-card>
			</v-col>
		</v-row>
	</div>
</template>

<script lang="ts">
import { Context } from '@nuxt/types';
import { Vue, Component } from 'nuxt-property-decorator';
import { Chainseeker } from 'chainseeker';
import * as cs from 'chainseeker/dist/types';

const MAX_LATEST_BLOCKS = 5;
const MAX_LATEST_TXS = 5;

interface CoinStatus {
	icon: any,
	blocks: number,
	lastBlock: cs.BlockHeader,
}

@Component({
	layout: 'base',
})
export default class Home extends Vue {
	status: { [key: string]: CoinStatus } = {};
	async asyncData({ params, error, $config }: Context) {
		const status: { [key: string]: CoinStatus } = {};
		for(const coin in $config.coins) {
			if(coin === 'local') continue;
			const icon = require(`~/assets/img/coins/${coin}.png`);
			const cs = new Chainseeker($config.coins[coin].apiEndpoint);
			try {
				const { blocks } = await cs.getStatus();
				const lastBlock = await cs.getBlockHeader(blocks);
				status[coin] = {
					icon,
					blocks,
					lastBlock,
				};
			} catch(e) {
				status[coin] = {
					icon,
				};
			}
		}
		return {
			status,
		};
	}
}
</script>

