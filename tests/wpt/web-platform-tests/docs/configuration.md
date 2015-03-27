Web-platform-tests are designed to run in a self-contained environment
on the local computer. All the required resources are packaged with
the web-platform-tests repository.

## Requirements

 * [git](http://git-scm.com/)
 * [Python 2.7](http://python.org)
 * [OpenSSL](https://www.openssl.org)

## Hosts configuration

The tests depend on certain domains being available. These are
typically configured locally with `web-platform.test` as the top level
domain and five subdomains. To configure these domains you need to edit
your
[`hosts` file](http://en.wikipedia.org/wiki/Hosts_%28file%29%23Location_in_the_file_system). The
following entries are required:

```
127.0.0.1	web-platform.test
127.0.0.1	www.web-platform.test
127.0.0.1	www1.web-platform.test
127.0.0.1	www2.web-platform.test
127.0.0.1	xn--n8j6ds53lwwkrqhv28a.web-platform.test
127.0.0.1	xn--lve-6lad.web-platform.test
```

## Cloning the Repository

If you have not done so, clone the web-platform-tests repository:

    git clone --recursive git@github.com:w3c/web-platform-tests.git

If you have already made a clone, but did not specify `--recursive`
update all submodules:

    git submodule update --init --recursive

## Font Files

Many layout tests require a set of test-specific fonts, notably
Ahem. These are available from the
[CSS Fonts](http://www.w3.org/Style/CSS/Test/Fonts/) website. These
must be installed according to the normal font-install procedure for
your operating system.

## Running the Test Server

The test environment can be started using

    ./serve

This will start HTTP servers on two ports and a websockets server on
one port. By default one web server starts on port 8000 and the other
ports are randomly-chosen free ports. Tests must be loaded from the
*first* HTTP server in the output. To change the ports, copy the
`config.default.json` file to `config.json` and edit the new file,
replacing the part that reads:

```
"http": [8000, "auto"]
```

to some port of your choice e.g.

```
"http": [1234, "auto"]
```

If you installed OpenSSL in such a way that running `openssl` at a
command line doesn't work, you also need to adjust the path to the
OpenSSL binary. This can be done by adding a section to `config.json`
like:

```
"ssl": {"openssl": {"binary": "/path/to/openssl"}}
```
