<template>
	<span>{{ unitInSatoshi ? value : (value * 1e-8).toFixed(8) }} <small>{{ symbol ? symbol : $config.coin.symbol }}</small></span>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'nuxt-property-decorator';

@Component
export default class Amount extends Vue {
	@Prop({ type: Number, required: true, })
	value!: number;
	@Prop({ type: String, required: false, default: '' })
	symbol!: string;
	@Prop({ type: Boolean, required: false, default: false })
	unitInSatoshi!: boolean;
}
</script>
