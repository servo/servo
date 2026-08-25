#!/bin/sh

INPUT=wav.wav
sox -V -r 44100 -n -b 8 -c 1 wav.wav synth 0.01 sin 330 vol -6db
ffmpeg -i $INPUT -write_xing 0 -id3v2_version 0 mp3-raw.mp3
ffmpeg -i $INPUT mp3-with-id3.mp3
ffmpeg -i $INPUT flac.flac
ffmpeg -i $INPUT ogg.ogg
ffmpeg -i $INPUT mp4.mp4
ffmpeg -i $INPUT webm.webm
