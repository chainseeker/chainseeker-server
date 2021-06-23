#!/bin/bash

if [ $# -le 0 ]; then
	echo "usage: $0 ADDRESS"
	exit
fi

ADDRESS=$1

B="bitcoin-cli -regtest"

HEIGHT=$($B getblockcount)
echo "Best block height = ${HEIGHT}"
BESTHASH=$($B getblockhash ${HEIGHT})
echo "Best block hash = ${BESTHASH}"
$B invalidateblock ${BESTHASH}
$B generatetoaddress 1 ${ADDRESS}

