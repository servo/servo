name: manifest
on:
  push:
    branches:
      - master
  pull_request:
    paths:
      - 'tools/**'
jobs:
  build-and-tag:
    runs-on: ubuntu-20.04
    steps:
    - name: Set up Python
      uses: actions/setup-python@v3
      with:
        python-version: '3.10'
    - name: Checkout
      uses: actions/checkout@v3
      with:
        fetch-depth: 50
    - name: Install dependencies
      run: |
        sudo apt-get -qqy install zstd
        pip install -r tools/wpt/requirements.txt
    - name: Run manifest_build.py
      # Use a conditional step instead of a conditional job to work around #20700.
      if: github.repository == 'web-platform-tests/wpt'
      run: tools/docker/retry.py --delay 60 python tools/ci/manifest_build.py
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
