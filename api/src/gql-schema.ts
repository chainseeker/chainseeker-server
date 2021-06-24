
const schema = `
	type Status {
		blocks: Int!
	}
	type CounterParty {
		destination: String!
		message: String!
	}
	type ScriptSig {
		asm: String!
		hex: String!
	}
	type Vin {
		txid: String!
		vout: String!
		scriptSig: ScriptSig!
		txinwitness: [String]!
		sequence: String!
		value: String!
		address: String
	}
	type ScriptPubKey {
		asm: String!
		hex: String!
		type: String!
		address: String
	}
	type Vout {
		value: String!
		n: Int!
		scriptPubKey: ScriptPubKey!
	}
	type Transaction {
		hex: String!
		txid: String!
		hash: String!
		size: Int!
		vsize: Int!
		version: Int!
		locktime: Int!
		confirmed_height: Int!
		counterparty: CounterParty
		vin: [Vin]!
		vout: [Vout]!
		fee: String!
	}
	type Block {
		header: String!
		hash: String!
		version: Int!
		previousblockhash: String!
		merkleroot: String!
		time: Int!
		bits: String!
		difficulty: Float!
		nonce: String!
		size: Int!
		strippedsize: Int!
		weight: Int!
		height: Int!
		txids: [String]!
	}
	type Utxo {
		txid: String!
		vout: Int!
		scriptPubKey: ScriptPubKey!
		value: String!
	}
	type BlockSummary {
		hash: String!
		time: Int!
		nonce: String!
		size: Int!
		strippedsize: Int!
		weight: Int!
		txcount: Int!
	}
	type AddressBalance {
		scriptPubKey: String!
		address: String
		value: String!
	}
	type AddressBalanceResponse {
		count: Int!,
		data: [AddressBalance]!
	}
	type Query {
		status: Status
		block(id: ID!): Block
		tx(id: ID!): Transaction
		txids(addr: ID!): [String]!
		utxos(addr: ID!): [Utxo]!
		blockSummary(offset: Int!, limit: Int!): [BlockSummary]!
		addressBalances(offset: Int!, limit: Int!): AddressBalanceResponse!
	}
`;

export default schema;

