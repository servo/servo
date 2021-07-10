% include base.dockerfile

RUN \
    apt-get install -qy --no-install-recommends \
        #
        # Multiple Android-related tools are in Java
        openjdk-8-jdk-headless \
        #
        # Emulator dependencies
        libgl1 \
        libpulse0
