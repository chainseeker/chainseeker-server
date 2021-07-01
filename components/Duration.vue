<template>
	<span>
		{{ format(duration) }}
	</span>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'nuxt-property-decorator';

@Component
export default class Duration extends Vue {
	@Prop({ type: Number, required: true, })
	duration!: number;
	format(time: number) {
		const appendS = (n: number, text: string) => {
			return n > 1 ? `${text}s` : text;
		};
		const abs = Math.abs(time);
		// 0 ~ 1sec
		if(abs < 1000) {
			return `${time} ms`;
		}
		// 1sec ~ 1min
		if(abs < 60 * 1000) {
			const sec = Math.floor(time / 1000);
			return `${sec} ${appendS(sec, 'sec')}`;
		}
		// 1min ~ 1hour
		if(abs < 60 * 60 * 1000) {
			const min = Math.floor(time / 60 / 1000);
			const sec = Math.floor((time % (60 * 1000)) / 1000);
			return `${min} ${appendS(min, 'min')} ${sec} ${appendS(sec, 'sec')}`;
		}
		// 1hour ~ 1day
		if(abs < 24 * 60 * 60 * 1000) {
			const hour = Math.floor(time / 60 / 60 / 1000);
			const min = Math.floor((time % (60 * 60 * 1000)) / 60 / 1000);
			return `${hour} ${appendS(hour, 'hour')} ${min} ${appendS(min, 'min')}`;
		}
		// 1day ~
		const day = Math.floor(time / 24 / 60 / 60 / 1000);
		const hour = Math.floor((time % (24 * 60 * 60 * 1000)) / 60 / 60 / 1000);
		return `${day} ${appendS(day, 'day')} ${hour} ${appendS(hour, 'hour')}`;
	}
}
</script>
