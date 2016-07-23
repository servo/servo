.PHONY: all
all: ffmpeg
	@:  # No-op to silence the "make: Nothing to be done for 'all'." message.

.PHONY: ffmpeg
ffmpeg: ffmpeg-libs

ffmpeg-libs: ffmpeg-libs/Configure
	./ffmpeg.sh ${ANDROID_NDK}

ffmpeg-libs/Configure:
	wget https://www.guillaume-gomez.fr/ffmpeg-android.tar.gz
	tar -zxf ffmpeg-android.tar.gz
