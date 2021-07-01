<template>
	<div class="my-4">
		<div class="text-center">
			<v-btn-toggle v-model="active"><v-btn>Toggle API View</v-btn></v-btn-toggle>
		</div>
		<div v-if="typeof active !== 'undefined'" style="margin: 10px 0px;">
			<v-form v-on:submit="call">
				<v-text-field :prefix="`${$config.apiEndpoint}/v1/`" label="API URL" :value="path" />
				<div class="text-center">
					<v-btn type="submit">Call API</v-btn>
				</div>
			</v-form>
			<pre style="margin: 10px 0px; width: 100%;"><code style="display: block; width: 100%; padding: 10px; overflow-x: scroll;">{{ output }}</code></pre>
		</div>
	</div>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'nuxt-property-decorator';

@Component
export default class API extends Vue {
	@Prop({ type: String, required: true, })
	path: string;
	active?: number = 0;
	output: string = "(Please press the \"Call\" button...)";
	created() {
		this.active = undefined;
	}
	async call(e) {
		e.preventDefault();
		const url = `${this.$config.apiEndpoint}/v1/${this.path}`;
		const json = await (await fetch(url)).json();
		this.output = JSON.stringify(json, null, 4);
		return false;
	}
}
</script>
