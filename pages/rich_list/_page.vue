<template>
	<div v-if="richList">
		<h1>
				Address Rich List
				<small>({{ (page * PER_PAGE + 1).toLocaleString() }} - {{ Math.min(richListCount, (page + 1) * PER_PAGE).toLocaleString() }})</small>
		</h1>
		<div class="text-center">
			<v-pagination total-visible=10 :value="page + 1" :length="Math.ceil(richListCount / PER_PAGE)"
				v-on:input="(page) => $router.push('/rich_list/' + (+page - 1))" />
		</div>
		<v-simple-table>
			<template v-slot:default>
				<thead>
					<tr>
						<th>Rank</th>
						<th>Address</th>
						<th>Value</th>
					</tr>
				</thead>
				<tbody>
					<tr v-for="(item, i) in richList">
						<td>#{{ (page * PER_PAGE + i + 1).toLocaleString() }}</td>
						<td>
							<div v-if="item.scriptPubKey.address">
								<Address :value="item.scriptPubKey.address" />
							</div>
							<div v-else>
								<Address :value="item.scriptPubKey.hex" :display="`0x${item.scriptPubKey.hex.slice(0, 32)}...`" />
							</div>
						</td>
						<td>
							<Amount :value="item.value" /></NuxtLink>
						</td>
					</tr>
				</tbody>
			</template>
		</v-simple-table>
	</div>
</template>

<script lang="ts">
import { Context } from '@nuxt/types';
import { Vue, Component } from 'nuxt-property-decorator';
import { Chainseeker } from 'chainseeker';
import * as cs from 'chainseeker/dist/types';

const PER_PAGE = 100;

@Component
export default class Home extends Vue {
	PER_PAGE = PER_PAGE;
	page: number | null = null;
	richListCount: number | null = null;
	richList: cs.RichListEntry[] = [];
	data() {
		return {
			richListHeaders: [
				{ text: 'Address', value: 'address' },
				{ text: 'Value', value: 'value' },
			],
		};
	}
	head() {
		return { title: `Rich List ${this.page! * PER_PAGE + 1} - ${(this.page! + 1) * PER_PAGE} - chainseeker` };
	}
	async asyncData({ params, error, $config }: Context) {
		const page = typeof params.page === 'undefined' ? 0 : Number.parseInt(params.page);
		const cs = new Chainseeker($config.apiEndpoint);
		const richListCount = await cs.getRichListCount();
		const offset = page * PER_PAGE;
		if(offset > richListCount) {
			error({ statusCode: 404, message: 'The page has an invalid range.' });
			return;
		}
		const richList = await cs.getRichList(offset, PER_PAGE);
		return {
			page,
			richListCount,
			richList,
		};
	}
}
</script>

