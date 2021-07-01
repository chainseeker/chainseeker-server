<template>
	<span>
		<Duration :duration="elapsedTime" />
	</span>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'nuxt-property-decorator';

@Component
export default class ElapsedTime extends Vue {
	@Prop({ type: Number, required: true, })
	time!: number;
	elapsedTime: number = 0;
	constructor() {
		super();
		setInterval(this.update, 100);
	}
	update() {
		this.elapsedTime = Date.now() - this.time;
	}
}
</script>
