#!/bin/bash

if [ $# -le 0 ]; then
	echo usage: ./plot-nonce.sh CONFIG_NAME
	exit 1
fi

CONFIG_NAME=$1

TMP=$(mktemp)

DIR=$(cd $(dirname $0); pwd)
ts-node $DIR/src/fetch-nonce.ts $CONFIG_NAME >$TMP

DATA_POINTS=$(cat $TMP | wc -l)
if   [ $DATA_POINTS -ge 1000000 ]; then
	GNUPLOT_PS=0.1
elif [ $DATA_POINTS -ge 100000 ]; then
	GNUPLOT_PS=0.2
else
	GNUPLOT_PS=0.4
fi

gnuplot -e " \
	set terminal png size 1920,1080 font 'VL P Gothic,18'; \
	set xlabel 'Block Height'; \
	set ylabel 'Nonce'; \
	set grid xtics ytics mxtics mytics; \
	set nokey; \
	plot '-' using 1:2 with points pt 7 ps ${GNUPLOT_PS} lc rgb 'black'; \
" <$TMP

rm $TMP

