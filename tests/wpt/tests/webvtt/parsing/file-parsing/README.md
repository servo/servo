# Tests for http://w3c.github.io/webvtt/#file-parsing

Tests that expect an 'error' event (due to invalid signature) are:

```bash
./signature-invalid.html
./support/*.vtt
```

Other tests are generated from source files with a custom format. The source files are:

```bash
./support/*.test
```

## .test Format

* The first line is the title of the test.
* Subsequent lines until a blank line contain HTML metadata.
* Subsequent lines until a `===` line contains JS assertions.
* Finally the WebVTT file. Special characters can be escaped using python3 escape sequences: `\x00`, `\r`.

## Building Tests

Requirements: Python 3.2 or newer

```bash
$ python3 tools/build.py
```

## Spec Coverage Report

There is also a python implementation of the WebVTT file parser algorithm and a
script to create a test coverage report of this implementation, under `tools/`.

Requirements:
* Python 3.2 or newer
* [Coverage.py](https://pypi.python.org/pypi/coverage)

Installing Coverage.py using [pip](https://pypi.python.org/pypi/pip).

```bash
$ pip3 install coverage
```

Generating the report:

```bash
$ python3 spec_report.py
```

Will output `report.html`.
