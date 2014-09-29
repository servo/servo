CARGO_OPTS ?=

.PHONY: all
all: glut_app
	NDK_DEBUG=1 $(ANDROID_NDK)/ndk-build -B
	find glut_app/target ! \( -type d -name dist -prune \) -name libmozjs.so | \
		xargs -I {} cp -f {} libs/armeabi
	find glut_app/target ! \( -type d -name dist -prune \) -name 'libglut_app-*.so' | \
		xargs -I {} cp -f {} libs/armeabi/libservo.so
	find ../../rust/lib/rustlib/arm-linux-androideabi/lib \
		-name '*.so' -type f -size +1c | \
		xargs -I {} cp -f {} libs/armeabi
	$(ANDROID_SDK)/tools/android update project \
		--name ServoAndroid \
		--target "android-18" \
		--path .
	ant debug

.PHONY: glut_app
glut_app:
	cd glut_app; \
	../../../mach cargo build --target=arm-linux-androideabi $(CARGO_OPTS)

.PHONY: install
install:
	$(ANDROID_SDK)/platform-tools/adb install -r bin/ServoAndroid-debug.apk
