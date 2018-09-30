# Build a version of libgcal that includes commit
# https://gitlab.gnome.org/GNOME/gcab/commit/3365b4bd58dd7f13e786caf3c7234cf8116263d9
# which fixes "Invalid cabinet chunk" errors:
# https://gitlab.gnome.org/GNOME/msitools/issues/4#note_336695
FROM ubuntu:bionic-20180821
RUN \
    apt-get update -q && \
    apt-get install -qy --no-install-recommends \
        curl \
        ca-certificates \
        #
        # Build dependencies for libgcab
        meson \
        git \
        pkg-config \
        libc6-dev \
        libglib2.0-dev \
        libgirepository1.0-dev \
        gobject-introspection \
        valac \
        intltool \
    && \
    export REV=3365b4bd58dd7f13e786caf3c7234cf8116263d9 && \
    curl -L https://gitlab.gnome.org/GNOME/gcab/-/archive/$REV/gcab-$REV.tar.gz | tar -xz && \
    mv gcab-$REV gcab && \
    cd gcab && \
    meson build && \
    cd build && \
    # UTF-8 locale to work around https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=870310
    export LANG=C.UTF-8 && \
    ninja && \
    cp -v libgcab/libgcab* /usr/local/lib


# FIXME: uncomment this after we upgrade docker-worker
# to a version of Docker that supports multi-stage builds:

# # Start a new image without the build dependencies, only the compiled library
# FROM ubuntu:bionic-20180821
# COPY --from=0 /usr/local/lib/libgcab* /usr/local/lib/

RUN \
    apt-get update -q && \
    apt-get install -qy --no-install-recommends \
        curl \
        ca-certificates \
        msitools