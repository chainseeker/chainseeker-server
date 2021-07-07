<template>
	<v-app>
		<v-main>
			<v-container>
				<Nuxt />
				<Footer />
			</v-container>
		</v-main>
	</v-app>
</template>

<script lang="ts">
import { Vue, Component } from 'nuxt-property-decorator';

@Component({
	layout: '',
})
export default class BaseLayout extends Vue {
}
</script>

