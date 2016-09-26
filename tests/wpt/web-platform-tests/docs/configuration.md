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
127.0.0.1   web-platform.test
127.0.0.1   www.web-platform.test
127.0.0.1   www1.web-platform.test
127.0.0.1   www2.web-platform.test
127.0.0.1   xn--n8j6ds53lwwkrqhv28a.web-platform.test
127.0.0.1   xn--lve-6lad.web-platform.test
0.0.0.0     nonexistent-origin.web-platform.test
```

## Cloning the Repository

If you have not done so, clone the web-platform-tests repository:

    git clone --recursive git@github.com:w3c/web-platform-tests.git

If you have already made a clone, but did not specify `--recursive`
update all submodules:

    git submodule update --init --recursive

## Font Files

A number of tests rely upon a set of custom fonts, with
[Ahem](https://github.com/w3c/csswg-test/raw/master/fonts/ahem/ahem.ttf)
being required to be installed according to the normal font-install
procedure for your operating system. Other tests which require other
fonts explicitly state this and provide links to required fonts.

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

### Windows Notes

Running wptserve with SSL enabled on Windows typically requires
installing an OpenSSL distribution.
[Shining Light](https://slproweb.com/products/Win32OpenSSL.html)
provide a convenient installer that is known to work, but requires a
little extra setup.

After installation ensure that the path to OpenSSL is on your `%Path%`
environment variable.

Then set the path to the default OpenSSL configuration file (usually
something like `C:\OpenSSL-Win32\bin\openssl.cfg` in the server
configuration. To do this copy `config.default.json` in the
web-platform-tests root to `config.json`. Then edit the JSON so that
the key `ssl/openssl/base_conf_path` has a value that is the path to
the OpenSSL config file.
