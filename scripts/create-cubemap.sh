#!/bin/sh
#https://stackoverflow.com/questions/70145023/how-to-create-a-ktx2-cubemap-texture-from-separate-images-using-toktx

SOURCE=$1
DEST=$(basename $SOURCE .png).ktx2

ktx create --encode uastc --zstd 18 --format R8G8B8_SRGB \
	--assign-oetf srgb --assign-primaries bt709 --generate-mipmap \
	$SOURCE $DEST

