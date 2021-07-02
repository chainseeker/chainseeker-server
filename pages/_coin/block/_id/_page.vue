<template>
	<div v-if="block">
		<h1>Block <small>#{{ block.height.toLocaleString() }}</small></h1>
		<div class="text-center">
			<v-pagination total-visible=10 :value="block.height" :length="status.blocks"
				v-on:input="(height) => $router.push('/block/' + height)" />
		</div>
		<div class="my-8">
			<v-row>
				<v-col md=2><strong>Block ID</strong></v-col>
				<v-col md=10>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								{{ block.hash }}
							</span>
						</template>
						<span>The hash of the block header (reversed).</span>
					</v-tooltip>
				</v-col>
			</v-row>
			<v-row>
				<v-col md=2><strong>Previous Block ID</strong></v-col>
				<v-col md=10>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								<NuxtLink :to="`./${block.previousblockhash}`">{{ block.previousblockhash }}</NuxtLink>
							</span>
						</template>
						<span>The hash of the previous block mined.</span>
					</v-tooltip>
				</v-col>
			</v-row>
			<v-row>
				<v-col md=2><strong>Merkle Root</strong></v-col>
				<v-col md=10>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								{{ block.merkleroot }}
							</span>
						</template>
						<span>Merkle root of the transactions included in this block.</span>
					</v-tooltip>
				</v-col>
			</v-row>
			<v-row>
				<v-col md=2><strong>Time</strong></v-col>
				<v-col md=4>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								{{ new Date(1000 * block.time).toLocaleString() }}
							</span>
						</template>
						<span>The time of this block mined.</span>
					</v-tooltip>
				</v-col>
				<v-col md=2><strong>Version</strong></v-col>
				<v-col md=4>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								{{ block.version }} (0x{{ block.version.toString(16).padStart(8, '0') }})
							</span>
						</template>
						<span>The version bitmap.</span>
					</v-tooltip>
				</v-col>
			</v-row>
			<v-row>
				<v-col md=2><strong>Bits</strong></v-col>
				<v-col md=4>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								{{ block.bits }}
							</span>
						</template>
						<span>The difficulty bits.</span>
					</v-tooltip>
				</v-col>
				<v-col md=2><strong>Difficulty</strong></v-col>
				<v-col md=4>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								{{ block.difficulty }}
							</span>
						</template>
						<span>The difficulty target.</span>
					</v-tooltip>
				</v-col>
			</v-row>
			<v-row>
				<v-col md=2><strong>Nonce</strong></v-col>
				<v-col md=4>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								{{ block.nonce }} (0x{{ block.nonce.toString(16).padStart(8, '0') }})
							</span>
						</template>
						<span>Nonce</span>
					</v-tooltip>
				</v-col>
				<v-col md=2><strong>#transactions</strong></v-col>
				<v-col md=4>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								{{ block.txids.length.toLocaleString() }}
							</span>
						</template>
						<span>The number of transactions included in the block.</span>
					</v-tooltip>
				</v-col>
			</v-row>
			<v-row>
				<v-col md=2><strong>Size</strong></v-col>
				<v-col md=2>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								{{ block.size.toLocaleString() }} bytes
							</span>
						</template>
						<span>The size of the block (includes witness data).</span>
					</v-tooltip>
				</v-col>
				<v-col md=2><strong>Stripped Size</strong></v-col>
				<v-col md=2>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								{{ block.strippedsize.toLocaleString() }} bytes
							</span>
						</template>
						<span>The size of the block (exclude witness data).</span>
					</v-tooltip>
				</v-col>
				<v-col md=2><strong>Weight</strong></v-col>
				<v-col md=2>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								{{ block.weight.toLocaleString() }} WU
							</span>
						</template>
						<span>The block weight (4 * non-witness data + witness data).</span>
					</v-tooltip>
				</v-col>
			</v-row>
			<v-row>
				<v-col md=2><strong>Block Reward</strong></v-col>
				<v-col md=4>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								<Amount :value="coinbase.vout[0].value" />
							</span>
						</template>
						<span>Total claimed block reward.</span>
					</v-tooltip>
				</v-col>
				<v-col md=2><strong>Generated Coins</strong></v-col>
				<v-col md=4>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								<Amount :value="generatedAmount" />
							</span>
						</template>
						<span>Newly generated coins in the block.</span>
					</v-tooltip>
				</v-col>
			</v-row>
			<v-row>
				<v-col md=2><strong>Transaction Fee in Total</strong></v-col>
				<v-col md=10>
					<v-tooltip bottom>
						<template v-slot:activator="{ on, attrs }">
							<span v-bind="attrs" v-on="on">
								<Amount :value="fee" />
							</span>
						</template>
						<span>The total transaction fee.</span>
					</v-tooltip>
				</v-col>
			</v-row>
		</div>
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

