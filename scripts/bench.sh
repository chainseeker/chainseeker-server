#!/bin/bash

set -e
source "$(cd $(dirname $0) && pwd)/bench_common.sh"

if [ $# -lt 2 ]; then
	echo "Usage: $0 CONNECTIONS localhost:8000"
	exit
fi

CONNECTIONS=$1
ENDPOINT=$2

# Usage: run ENDPOINT
run() {
	echo "#### $1"
	echo '```'
	AUTOCANNON_FLAGS="--connections $CONNECTIONS" bench "$ENDPOINT/api/v1/$1"
	echo '```'
	echo
}

BLOCK_HEIGHT=500000
ADDRESS=bc1qgdjqv0av3q56jvd82tkdjpy7gdp9ut8tlqmgrpmv24sq90ecnvqqjwvw97

run status
run block/$BLOCK_HEIGHT
run block_with_txids/$BLOCK_HEIGHT
run block_with_txs/$BLOCK_HEIGHT
run tx/677b67a894d2587c423976ed65131d5ea730d9bd164e7692beffc0441f40eebf
run txids/$ADDRESS
run txs/$ADDRESS
run utxos/$ADDRESS
run rich_list_count
run rich_list/0/100
run rich_list_addr_rank/$ADDRESS

