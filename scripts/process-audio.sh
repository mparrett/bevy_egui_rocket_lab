#!/bin/sh

INFILE=assets/audio/Welcome_to_the_Lab_v1.mp3
OUTFILE=assets/audio/Welcome_to_the_Lab_v1.ogg
# Convert mp3 to ogg
ffmpeg -i $INFILE -c:a libvorbis -q:a 4 temp.ogg
# Normalize audio
ffmpeg -i temp.ogg -filter:a loudnorm -c:a libvorbis -q:a 4 $OUTFILE 
ffmpeg -i $OUTFILE temp.mp3  # mp3 for testing, can't easily play ogg
rm temp.ogg
afplay temp.mp3
