#!/bin/bash

set -e
source "$(cd $(dirname $0) && pwd)/bench_common.sh"

if [ $# -le 0 ]; then
	echo "Usage: $0 localhost:8332"
	exit
fi

ENDPOINT=$1

# Usage: run ENDPOINT
run() {
	echo "## $1"
	echo '```'
	bench "$ENDPOINT/rest/$1"
	echo '```'
	echo
}

echo "# Benchmark results for Bitcoin Core REST API"
echo
run chaininfo.json
run blockhashbyheight/500000.json
# height = 500000
run block/00000000000000000024fb37364cbf81fd49cc2d51c09c75c35433c3a1945d04.json
run tx/677b67a894d2587c423976ed65131d5ea730d9bd164e7692beffc0441f40eebf.json

