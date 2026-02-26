#!/bin/sh

INFILE=$1
OUTFILE=$(echo $INFILE | sed 's/\.png$/\.ktx2/')

read -p "Convert $INFILE to $OUTFILE? [y/N] " -n 1 -r

toktx --2d --genmipmap --target_type RGBA --t2 --encode etc1s --clevel 5 --qlevel 255 \
		--assign_oetf srgb $OUTFILE $INFILE
