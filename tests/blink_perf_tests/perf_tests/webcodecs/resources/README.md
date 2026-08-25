# WebCodecs Test Files

[TOC]

## Instructions

To add, update, or remove a test file, please update the list below.

## List of Test Files

### 720p.h264
```
ffmpeg -f lavfi -i testsrc=size=1280x720:rate=1:duration=1 -pix_fmt yuv420p -vcodec h264 -tune zerolatency -f h264 720p.h264
```
