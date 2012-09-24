DEPS_rust-azure += \
	rust-geom

DEPS_rust-glut += \
	rust-opengles

DEPS_rust-layers += \
	rust-geom \
	rust-opengles \
	rust-glut \
	rust-azure

DEPS_sharegl += \
	rust-opengles \
	rust-geom

DEPS_servo-sandbox += \
	libhubbub \
	libparserutils

DEPS_rust-hubbub += \
	libhubbub

CFLAGS_mozjs += \
	"-I../mozjs/dist/include"

DEPS_rust-mozjs += \
	mozjs

CFLAGS_rust-mozjs += \
	"-I../mozjs/dist/include"

# Platform-specific dependencies
ifeq ($(CFG_OSTYPE),darwin)
DEPS_rust-azure += \
	rust-cocoa \
	rust-core-foundation

DEPS_rust-layers += \
	rust-cocoa

DEPS_rust-io-surface += \
	rust-core-foundation

DEPS_sharegl += \
	rust-core-foundation \
	rust-io-surface
endif
