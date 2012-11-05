# NOTE: the make magic can only compute transitive build dependencies,
# not transitive link flags. So, if A -> B -> C, must add A as a dep
# of C so the correct -L/path/to/A flag is generated for building C.

# NB. This should not be a problem once a real package system exists.

DEPS_rust-azure += \
	rust-geom \
	rust-cairo \
	skia \
	$(NULL)

DEPS_rust-glut += \
	rust-opengles \
	$(NULL)

DEPS_rust-layers += \
	rust-azure \
	rust-cairo \
	rust-geom \
	rust-glut \
	rust-opengles \
	$(NULL)

DEPS_sharegl += \
	rust-geom \
	rust-opengles \
	$(NULL)

DEPS_rust-hubbub += \
	libhubbub \
	$(NULL)

DEPS_rust-netsurfcss += \
	libcss \
	rust-wapcaplet \
	$(NULL)

DEPS_rust-wapcaplet += \
	libwapcaplet \
	$(NULL)

CFLAGS_rust-wapcaplet += \
	"-I$(S)src/libwapcaplet/include" \
	$(NULL)

DEPS_rust-css += \
	rust-netsurfcss \
	rust-wapcaplet \
	$(NULL)

CFLAGS_mozjs += \
	"-I../mozjs/dist/include" \
	$(NULL)

DEPS_rust-mozjs += \
	mozjs \
	$(NULL)

CFLAGS_rust-mozjs += \
	"-I../mozjs/dist/include" \
	$(NULL)

DEPS_libcss += \
	libwapcaplet \
	libparserutils \
	$(NULL)

# Platform-specific dependencies
ifeq ($(CFG_OSTYPE),darwin)
DEPS_rust-azure += \
	rust-core-graphics \
	rust-core-foundation \
	$(NULL)

DEPS_rust-cairo += \
	rust-core-graphics \
	$(NULL)

DEPS_rust-io-surface += \
	rust-core-foundation \
	$(NULL)

DEPS_sharegl += \
	rust-core-foundation \
	rust-io-surface \
	$(NULL)

DEPS_rust-core-text += \
	rust-core-foundation \
	rust-core-graphics \
	$(NULL)

DEPS_rust-layers += \
	rust-core-graphics \
	$(NULL)

endif

ifeq ($(CFG_OSTYPE),linux)

DEPS_rust-cairo += \
	rust-freetype \
	rust-fontconfig \
	rust-xlib \
	$(NULL)

DEPS_rust-azure += \
	rust-freetype \
	rust-fontconfig \
	rust-xlib \
	$(NULL)

# See note at top of file
DEPS_rust-layers += \
	rust-freetype \
	rust-fontconfig \
	$(NULL)
endif
