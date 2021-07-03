<template>
	<v-app>
		<v-main>
			<v-container>
				<nav>
					<v-toolbar>
						<v-toolbar-title>
							<NuxtLink to="/" style="color: inherit; vertical-align: 0;">
								<img src="~/assets/img/logo-wide-small.png" alt="site logo" style="max-height: 40px;" />
							</NuxtLink>
							<span v-for="(config, coin) in $config.coins" :key="coin">
								<NuxtLink v-if="coin !== 'local'" :to="`/${coin}`" style="color: #666; font-size: 80%;" class="ml-2">
									<img :src="icons[coin]" :alt="`${config.coin.name} icon`" style="height: 5ex;" />
								</NuxtLink>
							</span>
						</v-toolbar-title>
						<div class="ml-4"><NuxtLink :to="`/${$route.params.coin}/rich_list`">Rich List</NuxtLink></div>
						<div class="ml-2"><a href="https://chainseeker.docs.apiary.io/" target="_blank">REST API</a></div>
						<v-spacer />
						<v-form v-on:submit="search">
							<v-container style="margin-top:2ex">
								<v-row>
									<v-col>
										<v-text-field v-model="query" label="blockid, height, txid, address" style="width: 20em" />
									</v-col>
									<v-col>
										<v-btn type="submit">Search</v-btn>
									</v-col>
								</v-row>
							</v-container>
						</v-form>
					</v-toolbar>
				</nav>
				<Nuxt />
				<Footer />
			</v-container>
		</v-main>
	</v-app>
</template>

<script lang="ts">
import { Vue, Component } from 'nuxt-property-decorator';

@Component({
	layout: 'base',
	middleware: ['coin'],
})
export default class Layout extends Vue {
	icons: any = {};
	query: string = '';
	async fetch() {
		for(const coin in this.$config.coins) {
			if(coin === 'local') continue;
			this.icons[coin] = require(`~/assets/img/coins/${coin}.png`);
		}
	}
	search(e: Event) {
		e.preventDefault();
		this.$router.push(`/${this.$config.coin}/search/${this.query}`);
		return false;
	}
}
</script>

