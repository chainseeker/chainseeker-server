#!/bin/bash

REST_ENDPOINT=http://localhost:18443/rest

if [ $# -le 0 ]; then
	echo "useage: $0 HEIGHT"
	exit
fi

HEIGHT=$1
echo "Resolving block hash at height = ${HEIGHT} ..."
BLOCK_HASH=$(curl "${REST_ENDPOINT}/blockhashbyheight/$1.json" 2>/dev/null | jq .blockhash | tr -d '"')
echo "Fetching block hash = ${BLOCK_HASH} ..."
curl "${REST_ENDPOINT}/block/${BLOCK_HASH}.bin" 2>/dev/null >block_$1.bin

