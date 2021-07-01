<template>
</template>

<script lang="ts">
import { Vue, Component } from 'nuxt-property-decorator';
import { Chainseeker } from 'chainseeker';
import * as cs from 'chainseeker/dist/types';

@Component
export default class Home extends Vue {
	async mounted() {
		const query = this.$route.params.query;
		const cs = new Chainseeker(this.$config.apiEndpoint);
		// Try to fetch as a block.
		try {
			const block = await cs.getBlockHeader(query);
			this.$router.push(`/block/${query}`);
			return;
		} catch(e) {
		}
		// Try to fetch as a transaction.
		try {
			const block = await cs.getTransaction(query);
			this.$router.push(`/tx/${query}`);
			return;
		} catch(e) {
		}
		// Try to fetch as an address.
		try {
			const block = await cs.getUtxos(query);
			this.$router.push(`/addr/${query}`);
			return;
		} catch(e) {
		}
		this.$nuxt.error({ statusCode: 404, message: 'Not Found.' });
	}
}
</script>

