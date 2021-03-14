name: CI

on:
  # Triggers the workflow on push or pull request events but only for the master branch
  push:
    branches: [ "master", "github-actions-dev", "auto", "try", "try-linux", "try-mac" ]
  pull_request:
    branches: [ "master", "github-actions-dev" ]

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
          curl -OLS https://github.com/wixtoolset/wix3/releases/download/wix3111rtm/wix311.exe
          .\wix311.exe /install /quiet /norestart
          python -m pip install --upgrade pip virtualenv
          python mach fetch
      - name: Release build
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: python mach build --release --media-stack=dummy
      - name: Unit tests
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: python mach test-unit --release
      - name: Package
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: python mach package --release
      - name: libsimpleservo
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: python mach build --release --libsimpleservo
      - name: Smoketest
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: python mach smoketest --angle

  build-win-uwp:
    name: Build (Windows UWP)
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
      - name: Tidy
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: python mach test-tidy --force-cpp --no-wpt

  build-win-arm64:
    name: Build (Windows UWP ARM64)
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
        run: python mach build --release --target=aarch64-uwp-windows-msvc
      - name: Package
        working-directory: "C:\\a\\${ REPOSITORY_NAME }\\${ REPOSITORY_NAME }"
        run: python mach package --release --target=aarch64-uwp-windows-msvc --uwp=arm64

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
      - name: Package binary
        run: tar -czf target.tar.gz target/release/servo resources
      - name: Archive binary
        uses: actions/upload-artifact@v2
        with:
          name: release-binary
          path: target.tar.gz

  build-linux-alt:
    name: Build (Linux non-default)
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
      - name: Tidy
        run: |
          python3 ./mach test-tidy --no-progress --all
          python3 ./mach test-tidy --no-progress --self-test
      # Dev build disabled due to OOM
      #- name: Dev build
      #  run: python3 ./mach build --dev
      - name: Lockfile check
        run: ./etc/ci/lockfile_changed.sh
      - name: Memory reports
        run: ./etc/memory_reports_over_time.py --test
      - name: Disallowed panics
        run: ./etc/ci/check_no_panic.sh
      - name: Unit tests
        run: python3 ./mach test-unit --release
      - name: Refcell backtrace
        run: python3 ./mach build --release --features refcell_backtrace
      - name: Layout 2020
        run: python3 ./mach build --release --features layout-2020
      - name: libsimpleservo
        run: python3 ./mach build --release --libsimpleservo
      - name: gstplugin
        run: python3 ./mach build --release -p servo-gst-plugin
      - name: Dummy media stack
        run: python3 ./mach build --release --media-stack=dummy

% for chunk in range(1, total_chunks + 1):
  # linux-wpt${chunk}:
  #  #needs: build-linux
  #  runs-on: ubuntu-20.04
  #  steps:
  #    - uses: actions/checkout@v2
  #      with:
  #        fetch-depth: 2

  #    #- name: Download release binary
  #    #  uses: actions/download-artifact@v2
  #    #  with:
  #    #    name: release-binary

  #    - name: Fake build
  #      run: |
  #        wget https://joshmatthews.net/release-binary.zip
  #        unzip release-binary.zip

  #    - name: Prep test environment
  #      run: |
  #        tar -xzf target.tar.gz
  #        python3 -m pip install --upgrade pip virtualenv
  #        sudo apt update
  #        sudo apt install -qy --no-install-recommends libgl1 libssl1.1 libdbus-1-3 libxcb-xfixes0-dev libxcb-shape0-dev libunwind8 libegl1-mesa
  #        wget http://mirrors.kernel.org/ubuntu/pool/main/libf/libffi/libffi6_3.2.1-8_amd64.deb
  #        sudo apt install ./libffi6_3.2.1-8_amd64.deb
  #        python3 ./mach bootstrap-gstreamer

  #    - name: Run tests
  #      run: |
  #        python3 ./mach test-wpt --release --processes=2 --total-chunks=${total_chunks} --this-chunk=${chunk} --log-raw=test-wpt.log --log-servojson=wpt-jsonsummary.log --always-succeed | cat
  #        python3 ./mach filter-intermittents wpt-jsonsummary.log --log-intermittents=intermittents.log --log-filteredsummary=filtered-wpt-summary.log --tracker-api=default --reporter-api=default

  #    - name: Archive logs
  #      uses: actions/upload-artifact@v2
  #      with:
  #        name: wpt${chunk}-logs-linux
  #        path: |
  #          test-wpt.log
  #          wpt-jsonsummary.log
  #          filtered-wpt-summary.log
  #          intermittents.log
% endfor

  build_result:
    name: homu build finished
    runs-on: ubuntu-latest
    needs: ["build-win", "build-mac", "build-linux", "build-linux-alt", "build-win-uwp", "build-win-arm64"]
    steps:
      - name: Mark the job as successful
        run: exit 0
        if: success()
      - name: Mark the job as unsuccessful
        run: exit 1
        if: "!success()"
