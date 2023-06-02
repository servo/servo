Always use `getResourcePath()` to get the appropriate path to these resources depending
on the context (WPT, standalone, worker, etc.)


The test video files were generated with the ffmpeg cmds below:
ffmpeg.exe -loop 1 -i .\four-colors.png -c:v libvpx -pix_fmt yuv420p -frames 500 -colorspace smpte170m -color_primaries smpte170m -color_trc smpte170m -color_range tv four-colors-vp8-bt601.webm
ffmpeg.exe -loop 1 -i .\four-colors.png -c:v libtheora -pix_fmt yuv420p -frames 500 -colorspace smpte170m -color_primaries smpte170m -color_trc smpte170m -color_range tv four-colors-theora-bt601.ogv
ffmpeg.exe -loop 1 -i .\four-colors.png -c:v libx264 -pix_fmt yuv420p -frames 500 -colorspace smpte170m -color_primaries smpte170m -color_trc smpte170m -color_range tv four-colors-h264-bt601.mp4
ffmpeg.exe -loop 1 -i .\four-colors.png -c:v libvpx-vp9 -pix_fmt yuv420p -frames 500 -colorspace smpte170m -color_primaries smpte170m -color_trc smpte170m -color_range tv four-colors-vp9-bt601.webm
ffmpeg.exe -loop 1 -i .\four-colors.png -c:v libvpx-vp9 -pix_fmt yuv420p -frames 500 -colorspace bt709 -color_primaries bt709 -color_trc bt709 -color_range tv -vf scale=out_color_matrix=bt709:out_range=tv four-colors-vp9-bt709.webm

These rotation test files are copies of four-colors-h264-bt601.mp4 with metadata changes.
ffmpeg.exe -i .\four-colors-h264-bt601.mp4 -c copy -metadata:s:v rotate=90 four-colors-h264-bt601-rotate-90.mp4
ffmpeg.exe -i .\four-colors-h264-bt601.mp4 -c copy -metadata:s:v rotate=180 four-colors-h264-bt601-rotate-180.mp4
ffmpeg.exe -i .\four-colors-h264-bt601.mp4 -c copy -metadata:s:v rotate=270 four-colors-h264-bt601-rotate-270.mp4