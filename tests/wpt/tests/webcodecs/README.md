# WebCodecs Test Files

[TOC]

## Instructions

To add, update or remove a test file, please update the list below.

Please provide full reference and steps to generate the test file so that
any people can regenerate or update the file in the future.

## Notes
* When updating the sample offsets and descriptions for tests using mp4 files, it's easiest to use [mp4box.js](https://gpac.github.io/mp4box.js/test/filereader.html).
  * Sample offsets can be copied from the "Sample View" tab after unchecking all but offset and size. Use a multi-line edit mode and clang-format to quickly format entries.
  * Description entries can be found under moov.trak.mdia.minf.stbl.stsd in box view.
    * avc1.avcC or hvc1.hvcC has an offset, size in the same view. Add 8 to offset and subtract 8 from the size to get the values the tests want.
  * If you use ffprobe -show_packets to get sample offsets, you may need to add 4 to each `pos` value. You can tell if you need to by whether or not tests pass.

## List of Test Files

### four-colors.png
Generated using MSPaint like a true professional.

### four-colors.avif
Lossless encoding must be used to ensure colors are perfect.
```
avifenc -l four-colors.png -o four-colors.avif
```

### four-colors.webp
Lossless encoding must be used to ensure colors are perfect.
```
ffmpeg -i four-colors.png -lossless 1 -y four-colors.webp
```

### four-colors-limited-range-420-8bpc.webp
```
ffmpeg -i four-colors.png -pix_fmt yuv420p four-colors-limited-range-420-8bpc.webp
```

### four-colors.gif
High quality encoding must be used to ensure colors are perfect.
```
cp four-colors.png four-colors2.png
gifski -o four-colors.gif four-colors*.png
```

### four-colors-flip.gif
High quality encoding must be used to ensure colors are perfect.
```
ffmpeg -i four-colors.png -vf "rotate=PI" four-colors2.png
gifski -o four-colors-flip.gif four-colors*.png
```

### four-colors-flip.avif
```
ffmpeg -i four-colors-flip.gif -vcodec libaom-av1 -crf 16 four-colors-flip.mp4
mp4box -add-image ref:primary:tk=1:samp=1 -ab avis -ab avif -ab miaf -brand avis four-colors-flip.mp4 -out four-colors-flip.avif
mp4box -edits 1=r four-colors-flip.avif
```

### four-colors-limited-range-(420|422|444)-8bpc.avif
```
avifenc -r l -d 8 -y 420 -s 0 four-colors.png four-colors-limited-range-420-8bpc.avif
avifenc -r l -d 8 -y 422 -s 0 four-colors.png four-colors-limited-range-422-8bpc.avif
avifenc -r l -d 8 -y 444 -s 0 four-colors.png four-colors-limited-range-444-8bpc.avif
```

### four-colors-full-range-bt2020-pq-444-10bpc.avif
```
avifenc -r f -d 10 -y 444 -s 0 --nclx 9/16/9 four-colors.png four-colors-full-range-bt2020-pq-444-10bpc.avif
```

### four-colors.jpg
Used [Sqoosh.app](https://squoosh.app/) with MozJPEG compression and RGB
channels. exiftool was then used to add an orientation marker.
```
exiftool -Orientation=1 -n four-colors.jpg
```

### four-colors-limited-range-420-8bpc.jpg
Used [Sqoosh.app](https://squoosh.app/) with MozJPEG compression and YUV
channels. exiftool was then used to add an orientation marker.
```
exiftool -Orientation=1 -n four-colors-limited-range-420-8bpc.jpg
```

### four-colors.mp4
Used a [custom tool](https://storage.googleapis.com/dalecurtis/avif2mp4.html) to convert four-colors.avif into a .mp4 file.

### h264.mp4
```
ffmpeg -f lavfi -i testsrc=rate=10:n=1 -t 1 -pix_fmt yuv420p -vcodec h264 -tune zerolatency h264.mp4
```

### h264.annexb
```
ffmpeg -i h264.mp4 -codec copy -bsf:v h264_mp4toannexb -f h264 h264.annexb
```

### h265.mp4
```
ffmpeg -f lavfi -i testsrc=rate=10:n=1 -t 1 -pix_fmt yuv420p -vcodec hevc -tag:v hvc1 -tune zerolatency h265.mp4
```

### h265.annexb
```
ffmpeg -i h265.mp4 -codec copy -bsf:v hevc_mp4toannexb -f hevc h265.annexb
```

### sfx.adts
```
sox -n -r 48000 sfx.wav synth 1 sine 480
ffmpeg -i sfx.wav -frames:a 10 -acodec aac -b:a 96K sfx.adts
```

### sfx.mp3
```
sox -n -r 48000 sfx.wav synth 1 sine 480
ffmpeg -i sfx.wav -frames:a 10 -acodec libmp3lame -b:a 96K sfx.mp3
```

### sfx.flac
```
sox -n -r 48000 sfx.wav synth 1 sine 480
ffmpeg -i sfx.wav -frames:a 10 sfx.flac
```

### sfx-aac.mp4
```
sox -n -r 48000 sfx.wav synth 1 sine 480
ffmpeg -i sfx.wav -frames:a 10 -acodec aac -b:a 96K sfx-aac.mp4
```

### sfx-*.wav
```
sox -n -r 48000 sfx.wav synth 1 sine 480
for codec in s16 s24 s32 f32
do
  # Add "le" suffix
  ffmpeg -i sfx.wav -frames:a 10 -acodec pcm_"$codec"le sfx-pcm-$codec.wav
done
ffmpeg -i sfx.wav -frames:a 10 -acodec pcm_u8 sfx-pcm-u8.wav
for codec in alaw mulaw
do
  ffmpeg -i sfx.wav -frames:a 10 -acodec pcm_$codec sfx-$codec.wav
done
```

### sfx-opus.ogg
```
sox -n -r 48000 sfx.wav synth 1 sine 480
ffmpeg -i sfx.wav -frames:a 10 -acodec libopus -b:a 96K sfx-opus.ogg

### sfx-vorbis.ogg
```
sox -n -r 48000 sfx.wav synth 1 sine 480
ffmpeg -i sfx.wav -frames:a 10 -acodec libvorbis -b:a 96K sfx-vorbis.ogg
```

### av1.mp4
```
ffmpeg -f lavfi -i testsrc=rate=10:n=1 -t 1 -pix_fmt yuv420p -vcodec libaom-av1 av1.mp4
```

### vp8.webm
```
ffmpeg -f lavfi -i testsrc=rate=10:n=1 -t 1 -pix_fmt yuv420p -vcodec vp8 vp8.webm
```

### vp9.mp4
```
ffmpeg -f lavfi -i testsrc=rate=10:n=1 -t 1 -pix_fmt yuv420p -vcodec vp9 vp9.mp4
```
