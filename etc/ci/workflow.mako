name: CI

on:
  # Triggers the workflow on push events but only for the master branch
  push:
    branches: [ "auto", "try", "try-linux", "try-mac", "try-windows", "try-wpt"]

  # Allows you to run this workflow manually from the Actions tab
  workflow_dispatch:

env:
  RUST_BACKTRACE: 1
  SHELL: /bin/bash

jobs:
  build-win:
    name: Build (Windows)
    runs-on: windows-2019
    steps:
      - uses: actions/checkout@v2
        with:
          fetch-depth: 2
      - name: Copy to C drive
        run: cp D:\a C:\ -Recurse
      - name: Bootstrap
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: |
          python -m pip install --upgrade pip virtualenv
          python mach fetch
      - name: Release build
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: python mach build --release --media-stack=dummy
      - name: Unit tests
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: python mach test-unit --release
      - name: Smoketest
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: python mach smoketest --angle

  build-uwp-x64:
    name: Build (Windows UWP x64)
    runs-on: windows-2019
    steps:
      - uses: actions/checkout@v2
        with:
          fetch-depth: 2
      - name: Copy to C drive
        run: cp D:\a C:\ -Recurse
      - name: Bootstrap
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: |
          python -m pip install --upgrade pip virtualenv
          python mach fetch
      - name: Release build
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: python mach build --release --target=x86_64-uwp-windows-msvc
      - name: Package
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: python mach package --release --target=x86_64-uwp-windows-msvc --uwp=x64
        #env:
        #  CODESIGN_CERT: ${{ CODESIGN_CERT }}
      - name: Tidy
        run: python mach test-tidy --force-cpp --no-wpt

  build-uwp-arm64:
    name: Build (Windows UWP arm64)
    runs-on: windows-2019
    steps:
      - uses: actions/checkout@v2
        with:
          fetch-depth: 2
      - name: Copy to C drive
        run: cp D:\a C:\ -Recurse
      - name: Bootstrap
        working-directory: "C:\\a\\servo\\servo"
        run: |
          python -m pip install --upgrade pip virtualenv
          python mach fetch
      - name: Release build
        working-directory: "C:\\a\\servo\\servo"
        run: python mach build --release --target=aarch64-uwp-windows-msvc
      - name: Package
        working-directory: "C:\\a\\servo\\servo"
        run: python mach package --release --target=aarch64-uwp-windows-msvc --uwp=arm64
        #env:
        #  CODESIGN_CERT: ${{ CODESIGN_CERT }}

  build-mac:
    name: Build (macOS)
    runs-on: macos-10.15
    steps:
      - uses: actions/checkout@v2
        with:
          fetch-depth: 2
      - name: Bootstrap
        run: |
          python3 -m pip install --upgrade pip virtualenv
          brew bundle install --verbose --no-upgrade --file=etc/taskcluster/macos/Brewfile
          brew bundle install --verbose --no-upgrade --file=etc/taskcluster/macos/Brewfile-build
          rm -rf /usr/local/etc/openssl
          rm -rf /usr/local/etc/openssl@1.1
          brew install openssl@1.1 gnu-tar
      - name: Release build
        run: |
          export OPENSSL_INCLUDE_DIR="$(brew --prefix openssl)/include"
          export OPENSSL_LIB_DIR="$(brew --prefix openssl)/lib"
          export PKG_CONFIG_PATH="$(brew --prefix libffi)/lib/pkgconfig/"
          export PKG_CONFIG_PATH="$(brew --prefix zlib)/lib/pkgconfig/:$PKG_CONFIG_PATH"
          python3 ./mach build --release
      - name: Smoketest
        run: python3 ./mach smoketest
      - name: Unit tests
        run: python3 ./mach test-unit --release
      - name: Test package
        run: python3 ./mach package --release
      - name: Package smoketest
        run: ./etc/ci/macos_package_smoketest.sh target/release/servo-tech-demo.dmg
      - name: Package binary
        run: gtar -czf target.tar.gz target/release/servo target/release/*.dylib resources
      - name: Archive binary
        uses: actions/upload-artifact@v2
        with:
          name: release-binary-macos
          path: target.tar.gz

% for chunk in range(1, total_chunks + 1):
  # mac-wpt${chunk}:
  #   #needs: build-mac
  #   runs-on: macos-10.15
  #   steps:
  #     - uses: actions/checkout@v2
  #       with:
  #         fetch-depth: 2

  #     #- name: Download release binary
  #     #  uses: actions/download-artifact@v2
  #     #  with:
  #     #    name: release-binary-macos

  #     - name: Fake build
  #       run: |
  #         wget https://joshmatthews.net/release-binary-macos.zip
  #         unzip release-binary-macos.zip

  #     - name: Prep test environment
  #       run: |
  #         brew install gnu-tar
  #         gtar -xzf target.tar.gz
  #         python3 -m pip install --upgrade pip virtualenv
  #         brew bundle install --verbose --no-upgrade --file=etc/taskcluster/macos/Brewfile
  #     - name: Smoketest
  #       run: python3 ./mach smoketest
  #     - name: Run tests
  #       run: |
  #         python3 ./mach test-wpt --release --processes=3 --timeout-multiplier=8 --total-chunks=${total_chunks} --this-chunk=${chunk} --log-raw=test-wpt.log --log-servojson=wpt-jsonsummary.log --always-succeed | cat
  #         python3 ./mach filter-intermittents wpt-jsonsummary.log --log-intermittents=intermittents.log --log-filteredsummary=filtered-wpt-summary.log --tracker-api=default --reporter-api=default

  #     - name: Archive logs
  #       uses: actions/upload-artifact@v2
  #       with:
  #         name: wpt${chunk}-logs-macos
  #         path: |
  #           test-wpt.log
  #           wpt-jsonsummary.log
  #           filtered-wpt-summary.log
  #           intermittents.log
% endfor

  build-linux:
    name: Build (Linux)
    runs-on: ubuntu-20.04
    steps:
      - uses: actions/checkout@v2
        with:
          fetch-depth: 2
      - name: Bootstrap
        run: |
          python3 -m pip install --upgrade pip virtualenv
          sudo apt update
          python3 ./mach bootstrap
      - name: Release build
        run: python3 ./mach build --release
      - name: Lockfile check
        run: ./etc/ci/lockfile_changed.sh
      - name: Forbidden panic check
        run: ./etc/ci/check_no_panic.sh
      - name: Package binary
        run: tar -czf target.tar.gz target/release/servo resources
      - name: Archive binary
        uses: actions/upload-artifact@v2
        with:
          name: release-binary
          path: target.tar.gz

% for chunk in range(1, total_chunks + 1):
  linux-wpt-${chunk}:
    name: Linux WPT Tests ${chunk}
    runs-on: ubuntu-20.04
    needs: ["build-linux"]
    steps:
      - uses: actions/checkout@v2
        with:
          fetch-depth: 2
      - uses: actions/download-artifact@v2
        with:
          name: release-binary
          path: release-binary
      - name: unPackage binary
        run: tar -xzf release-binary/target.tar.gz
      - name: Prep test environment
        run: |
          python3 -m pip install --upgrade pip virtualenv
          sudo apt update
          sudo apt install -qy --no-install-recommends libgl1 libssl1.1 libdbus-1-3 libxcb-xfixes0-dev libxcb-shape0-dev libunwind8 libegl1-mesa
          wget http://mirrors.kernel.org/ubuntu/pool/main/libf/libffi/libffi6_3.2.1-8_amd64.deb
          sudo apt install ./libffi6_3.2.1-8_amd64.deb
          python3 ./mach bootstrap-gstreamer
      - name: Run tests
        run: |
          python3 ./mach test-wpt --release --processes $(nproc) --timeout-multiplier 2 --total-chunks ${total_chunks} --this-chunk ${chunk} --log-raw test-wpt.${chunk}.log --log-servojson wpt-jsonsummary.${chunk}.log --always-succeed
          python3 ./mach filter-intermittents wpt-jsonsummary.${chunk}.log --log-intermittents=intermittents.${chunk}.log --log-filteredsummary=filtered-wpt-summary.${chunk}.log --tracker-api=default --reporter-api=default
      - name: Archive logs
        uses: actions/upload-artifact@v2
        with:
          name: wpt${chunk}-logs-linux
          path: |
            test-wpt.${chunk}.log
            wpt-jsonsummary.${chunk}.log
            filtered-wpt-summary.${chunk}.log
            intermittents.${chunk}.log
% endfor

  build_result:
    name: homu build finished
    runs-on: ubuntu-latest
    needs:
      - "build-win"
      - "build-uwp-x64"
      - "build-uwp-arm64"
      - "build-linux"
      - "build-mac"
    % for chunk in range(1, total_chunks + 1):
      - "linux-wpt-${chunk}"
    % endfor

    steps:
      - name: Mark the job as successful
        run: exit 0
        if: success()
      - name: Mark the job as unsuccessful
        run: exit 1
        if: "!success()"
