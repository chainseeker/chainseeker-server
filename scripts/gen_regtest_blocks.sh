#!/bin/bash

B="bitcoin-cli -regtest"

OUTDIR=$(cd $(dirname $0)/../src/fixtures/regtest; pwd)

PRIV1="cMd16Mhym7Nbzr9LMdTjX13BkdfnSZzYdVzbvZVCuB9SB3mVansi"
ADDR1="bcrt1qjupnefcdghlx6jf3ppv7zv4fm7v0ja39dzzwvd"
PRIV2="cQsgk3rd6Yy1C4VVLPx4EduN3RQzpAqSXbwQSEXpB1hZeu8gGhNc"
ADDR2="bcrt1q7r57mcjte6cvzm7ua9fz08vqjnwxk06c2v6jdv"
PRIV3="cUVAkHac2bPhiJRm77nxFPj4TSejT3JzE8fhjmbtUfNUeA4Sfq2v"
ADDR3="bcrt1qzwashjmhdulpt75gqzrh25syf4xmy7uk6clm0p"

if [ $($B getblockcount) -ne 0 ]; then
	echo "The block count is not zero. Clear ~/.bitcoin/regtest and rerun."
	exit
fi

set -e
set -x

# fetch_block height file
fetch_block() {
	HEIGHT=$1
	FILE=$2
	BLOCKID=$($B getblockhash $HEIGHT)
	curl http://localhost:18443/rest/block/$BLOCKID.bin >$OUTDIR/$FILE.bin
}

$B createwallet default
$B importprivkey $PRIV1
$B importprivkey $PRIV2
$B importprivkey $PRIV3
# Generate 101 blocks.
$B generatetoaddress 101 $ADDR1
# Send 1 BTC to ADDR2.
$B -named sendtoaddress address=$ADDR2 amount=1 fee_rate=100
# Send 49 BTC to ADDR3.
$B -named sendtoaddress address=$ADDR3 amount=49 fee_rate=100
# Confirm the transactions.
$B generatetoaddress 1 $ADDR1
# Dump blocks.
for height in $(seq 0 102); do
	fetch_block $height block_$height
done
# Reorg the last block and generate a block rewarding to ADDR2.
$B invalidateblock $($B getblockhash 102)
$B generatetoaddress 1 $ADDR2
# Dump the highest block.
fetch_block 102 block_102_new

