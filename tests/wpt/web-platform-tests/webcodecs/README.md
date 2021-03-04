# WebCodecs Test Files

[TOC]

## Instructions

To add, update or remove a test file, please update the list below.

Please provide full reference and steps to generate the test file so that
any people can regenerate or update the file in the future.

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

### four-colors.gif
High quality encoding must be used to ensure colors are perfect.
```
cp four-colors.png four-colors2.png
gifski -o four-colors.gif four-colors*.png
```

### four-colors.jpg
Used [Sqoosh.app](https://squoosh.app/) with MozJPEG compression then used
exiftool to add an orientation marker.
```
exiftool -Orientation=1 -n four-colors.jpg
```
