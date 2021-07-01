<template>
	<span>
		<template v-if="!value">
			Unknown address
		</template>
		<template v-else>
			<span style="position:relative; top: 5px;">
			<span v-html="toSvg(value)"></span>
			</span>
			<NuxtLink :to="`/addr/${value}`">{{ value }}</NuxtLink>
		</template>
	</span>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'nuxt-property-decorator';
import { toSvg } from 'jdenticon';

@Component
export default class Address extends Vue {
	@Prop({ type: String, required: true, })
	value: string;
	toSvg(value: string): string {
		return toSvg(value, 20);
	}
}
</script>
