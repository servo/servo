.PHONY: all
all:
	NDK_DEBUG=1 $(ANDROID_NDK)/ndk-build -B
	find ../../target ! \( -type d -name dist -prune \) -name libmozjs.so | \
		xargs -I {} cp -f {} libs/armeabi
	find ../../target ! \( -type d -name dist -prune \) -name 'libservo-*.so' | \
		xargs -I {} cp -f {} libs/armeabi/libservo.so
	find ../../rust/lib/rustlib/arm-linux-androideabi/lib \
		-name '*.so' -type f -size +1c | \
		xargs -I {} cp -f {} libs/armeabi
	cp -a extlibs/* libs/armeabi
	$(ANDROID_SDK)/tools/android update project \
		--name ServoAndroid \
		--target "android-18" \
		--path .
	ant debug

.PHONY: install
install:
	$(ANDROID_SDK)/platform-tools/adb install -r bin/ServoAndroid-debug.apk
