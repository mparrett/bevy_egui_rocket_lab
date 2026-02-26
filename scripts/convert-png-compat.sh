#!/bin/sh

INFILE=$1
# to ktx2
OUTFILE=$(basename "$INFILE").ktx2

read -p "Convert $INFILE to $OUTFILE? [y/N] " -n 1 -r

toktx --2d --genmipmap --target_type RGBA --t2 \
	--encode astc --clevel 5 --qlevel 255 \
		--assign_oetf srgb "$OUTFILE" "$INFILE"
