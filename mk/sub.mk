NULL =

# Tests for these submodules will not be run by the default `make check` target.
SLOW_TESTS += \
	mozjs \
	$(NULL)

# Tests for these submodules do not exist.
NO_TESTS += \
	nspr \
	nss \
	$(NULL)

# These submodules will not be cleaned by the `make clean-fast` target.
SLOW_BUILDS += \
	libparserutils \
	mozjs \
	sharegl \
	skia \
	$(NULL)

# Builds that do not require rustc
NATIVE_BUILDS += \
	libhubbub \
	libparserutils \
	mozjs \
	skia \
	nss \
	nspr \
	glfw \
	$(NULL)

# NOTE: the make magic can only compute transitive build dependencies,
# not transitive link flags. So, if A -> B -> C, must add A as a dep
# of C so the correct -L/path/to/A flag is generated for building C.

# NB. This should not be a problem once a real package system exists.

DEPS_hubbub += \
	libparserutils \
	$(NULL)

DEPS_rust-azure += \
	rust-opengles \
	rust-layers \
	rust-geom \
	glfw-rs \
	glfw \
	skia \
	rust \
	$(NULL)

DEPS_glfw-rs += \
	glfw \
	rust \
	$(NULL)

DEPS_rust-layers += \
	rust-geom \
	rust-opengles \
	rust \
	$(NULL)

DEPS_sharegl += \
	rust-geom \
	rust-opengles \
	rust \
	$(NULL)

DEPS_rust-hubbub += \
	libhubbub \
	libparserutils \
	rust \
	$(NULL)

DEPS_rust-mozjs += \
	mozjs \
	rust \
	$(NULL)

CFLAGS_rust-mozjs += \
	"-I../mozjs/dist/include" \
	$(NULL)

DEPS_rust-png += \
	libpng \
	$(NULL)

# Platform-specific dependencies
ifeq ($(CFG_OSTYPE),apple-darwin)
DEPS_rust-azure += \
	rust-core-graphics \
	rust-core-text \
	rust-core-foundation \
	rust-cocoa \
	rust-io-surface \
	rust \
	$(NULL)

DEPS_rust-io-surface += \
	rust-core-foundation \
	rust-geom \
	rust-opengles \
	rust \
	$(NULL)

DEPS_rust-alert += \
	rust-core-foundation \
	rust-cocoa \
	rust \
	$(NULL)

DEPS_sharegl += \
	rust-core-foundation \
	rust-io-surface \
	rust \
	$(NULL)

DEPS_rust-core-graphics += \
	rust-core-foundation \
	rust \
	$(NULL)

DEPS_rust-core-text += \
	rust-core-foundation \
	rust-core-graphics \
	rust \
	$(NULL)

DEPS_rust-layers += \
	rust-core-foundation \
	rust-core-graphics \
	rust-core-text \
	rust-io-surface \
	rust-cocoa \
	rust \
	$(NULL)

endif

DEPS_nss += \
	nspr \
	$(NULL)

ifeq ($(CFG_OSTYPE),unknown-linux-gnu)

DEPS_rust-azure += \
	rust-freetype \
	rust-fontconfig \
	rust-xlib \
	rust \
	$(NULL)

# See note at top of file
DEPS_rust-layers += \
	rust-freetype \
	rust-fontconfig \
	rust-xlib \
	rust \
	$(NULL)
endif

ifeq ($(CFG_OSTYPE),linux-androideabi)
DEPS_rust-azure += \
	rust-freetype \
	rust-fontconfig \
	fontconfig \
	libfreetype2 \
	libexpat \
	rust \
	$(NULL)

# See note at top of file
DEPS_rust-layers += \
	rust-freetype \
	rust-fontconfig \
	rust-xlib \
	rust \
	$(NULL)

DEPS_rust-fontconfig += \
	fontconfig \
	rust-freetype \
	rust \
	$(NULL)

DEPS_rust-freetype += \
	libfreetype2 \
	rust \
	$(NULL)

DEPS_fontconfig += \
	libexpat \
	libfreetype2 \
	$(NULL)

CFLAGS_fontconfig += \
	"-I$(S)src/platform/android/libexpat/expat/lib" \
	"-I$(S)src/platform/android/libfreetype2/include" \
    $(NULL)

DEPS_skia += \
	libfreetype2 \
	$(NULL)

CXXFLAGS_skia += \
	-I$(S)src/platform/android/libfreetype2/include \
	$(NULL)

NATIVE_BUILD += \
	libfreetype2 \
	libexpat \
	fontconfig \
	$(NULL)
endif
