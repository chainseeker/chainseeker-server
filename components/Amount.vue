<template>
	<span>
		{{ (value * 1e-8).toFixed(8) }} <small>{{ $config.coin.symbol }}</small>
	</span>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'nuxt-property-decorator';

@Component
export default class Amount extends Vue {
	@Prop({ type: Number, required: true, })
	value: number;
}
</script>
