<template>
	<div class="my-4">
		<div><Address :value="address" /></div>
		<div class="ml-4">[ <Amount :value="amount" /> ]</div>
	</div>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'nuxt-property-decorator';
import * as cs from 'chainseeker/dist/types';

@Component
export default class TxMovement extends Vue {
	@Prop({ type: String | null, required: true, })
	address: string;
	@Prop({ type: Number, required: true, })
	amount: number;
}
</script>
