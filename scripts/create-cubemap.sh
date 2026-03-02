#!/bin/sh
#https://stackoverflow.com/questions/70145023/how-to-create-a-ktx2-cubemap-texture-from-separate-images-using-toktx

SOURCE=$1
DEST=$(basename $SOURCE .png).ktx2

ktx create --format ASTC_4x4_SRGB_BLOCK \
	--assign-oetf srgb --assign-primaries bt709 --generate-mipmap \
	$SOURCE $DEST

