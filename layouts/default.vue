<template>
	<v-app>
		<v-main>
			<v-container>
				<nav>
					<v-toolbar>
						<v-toolbar-title><NuxtLink to="/" style="color: inherit">chainseeker</NuxtLink></v-toolbar-title>
						<div class="ml-4"><NuxtLink to="/rich_list">Rich List</NuxtLink></div>
						<div class="ml-2"><a href="https://chainseeker.docs.apiary.io/" target="_blank">REST API</a></div>
						<v-spacer />
						<v-form>
							<v-container style="margin-top:2ex">
								<v-row>
									<v-col>
										<v-text-field v-model="query" label="blockid, height, txid, address" style="width: 40em" />
									</v-col>
									<v-col>
										<v-btn>Search</v-btn>
									</v-col>
								</v-row>
							</v-container>
						</v-form>
					</v-toolbar>
				</nav>
				<v-container>
					<Nuxt />
					<hr style="margin-top:5ex;" />
					<footer>
						<v-container>
							<p style="margin: 0px;">Copyright &copy; chainseeker 2017-{{ new Date().getFullYear() }}. All rights reserved.</p>
							<p>Created &amp; mainted by <NuxtLink to="https://twitter.com/visvirial" target="_blank">@visvirial</NuxtLink></p>
						</v-container>
					</footer>
				</v-container>
			</v-container>
		</v-main>
	</v-app>
</template>

<script lang="ts">
import { Vue, Component } from 'nuxt-property-decorator';

const a: string = 'false';

@Component({
	layout: 'default',
})
export default class Layout extends Vue {
	query: string = '';
}
</script>

