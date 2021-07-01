<template>
	<v-container>
		<v-row justify="space-between">
			<v-col md=5>
				<div v-for="vin in tx.vin">
					<AddressWithAmount :address="vin.address" :amount="vin.value" />
				</div>
			</v-col>
			<v-col md=1 class="text-center mt-8">
				<v-icon>mdi-arrow-right-bold-outline</v-icon>
			</v-col>
			<v-col md=5>
				<div v-for="vout in tx.vout">
					<AddressWithAmount :address="vout.scriptPubKey.address" :amount="vout.value" />
				</div>
			</v-col>
		</v-row>
	</v-container>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'nuxt-property-decorator';
import * as cs from 'chainseeker/dist/types';

@Component
export default class TxMovement extends Vue {
	@Prop({ type: Object, required: true, })
	tx!: cs.Transaction;
}
</script>
