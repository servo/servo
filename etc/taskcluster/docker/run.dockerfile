% include base.dockerfile

# Servo’s runtime dependencies:
RUN apt-get install -qy --no-install-recommends \
    libgl1 \
    libssl1.1 \
    libdbus-1-3 \
    libxcb-shape0-dev \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-libav \
    gstreamer1.0-gl \
    libunwind8
