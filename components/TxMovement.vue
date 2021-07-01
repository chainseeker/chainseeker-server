<template>
	<v-container>
		<v-row justify="space-between">
			<v-col md=5>
				<div v-for="vin in tx.vin">
					<AddressWithAmount :address="vin.address" :amount="vin.value" />
				</div>
			</v-col>
			<v-col md=1 class="text-center mt-8">
				<v-icon>mdi-arrow-right-bold-outline</v-icon>
			</v-col>
			<v-col md=5>
				<div v-for="vout in tx.vout">
					<AddressWithAmount :address="vout.scriptPubKey.address" :amount="vout.value" />
				</div>
			</v-col>
		</v-row>
	</v-container>
	<!--
	<% for(let j=0; j<Math.max(tx.vin.length, tx.vout.length); j++) { %>
	<tr>
		<td style="width:45%;">
			<% if(tx.vin[j]) { %>
				<% if(tx.vin[j].address != 'coinbase') { %>
					<div><%- formatAddress(tx.vin[j].address, colorize) %></div>
					<div style="margin-left:2em;">[<%- formatAmount(tx.vin[j].value) %>]</div>
				<% } else { /* Coinbase */ %>
					(coinbase transaction)
				<% } %>
			<% } %>
		</td>
		<td><%- j==0 ? '<i class="fa fa-arrow-circle-right"></i>' : '' %></td>
		<td style="width:45%;">
			<% if(tx.vout[j]) { %>
			<div>
				<%- formatAddress(tx.vout[j].scriptPubKey.address, colorize) %>
			</div>
			<div style="margin-left:2em;">[<%- formatAmount(tx.vout[j].value) %>]</div>
			<% } %>
		</td>
	</tr>
	<% } %>
	-->
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'nuxt-property-decorator';
import * as cs from 'chainseeker/dist/types';

@Component
export default class TxMovement extends Vue {
	@Prop({ type: Object, required: true, })
	tx: cs.Transaction;
}
</script>
