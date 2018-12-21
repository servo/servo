% include base.dockerfile

# Servo’s runtime dependencies
RUN apt-get install -qy --no-install-recommends \
    libgl1 \
    libssl1.0.0 \
    libdbus-1-3 \
    libgstreamer-plugins-bad1.0-0 \
    gstreamer1.0-plugins-good

