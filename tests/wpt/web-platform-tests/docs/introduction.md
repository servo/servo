---
layout: page
title: Introduction
---

web-platform-tests is a W3C-coordinated effort to build a
cross-browser testsuite for the majority of
the [web platform][web-platform]; it excludes only ECMAScript (whose
testsuite lives in [test262][test262]) and WebGL (whose testsuite
lives in [WebGL][WebGL]).

## Help!

If you get stuck or want clarification about anything, feel free to
ask on either the [mailing list][public-test-infra] or [IRC][]
([webclient][web irc], join channel `#testing`); IRC is generally
busiest during the European working day but frequently has people on
it at all times and should probably be the general first port of call
for any help.


## Testsuite Design

The vast majority of the testsuite is formed of HTML pages, which can
be loaded in a browser and either programmatically provide a result or
provide a set of steps to run the test and obtain the result.

The tests are, in general, short, cross-platform, and self-contained,
and should be easy to run in any browser.


## Test Layout

Each top level directory in the repository corresponds to tests for a
single specification. For W3C specs, these directories are typically
named after the shortname of the spec (i.e. the name used for snapshot
publications under `/TR/`); for WHATWG specs, they are typically named
after the subdomain of the spec (i.e. trimming `.spec.whatwg.org` from
the URL); for other specs, something deemed sensible is used. In any
case, there are occasional exceptions for historic reasons.

Within the specification-specific directory there are two common ways
of laying out tests: the first is a flat structure which is sometimes
adopted for very short specifications; the alternative is a nested
structure with each subdirectory corresponding to the id of a heading
in the specification. The latter provides some implicit metadata about
the part of a specification being tested according to its location in
the filesystem, and is preferred for larger specifications.


## Test Types

The testsuite has a few types of tests, outlined below:

* [testharness.js][] tests, which are run
  through a JS harness and report their result back with JS.

* [Reftests][], which render two (or more) web
  pages and combine them with equality assertions about their
  rendering (e.g., `A.html` and `B.html` must render identically), run
  either by the user switching between tabs/windows and trying to
  observe differences or through automated scripts.

* [Visual tests][visual] which display a page where the
  result is determined either by a human looking at it or by comparing
  it with a saved screenshot for that user agent on that platform.

* [Manual tests][manual], which rely on a human to run
  them and determine their result.

* WebDriver tests, which are used for testing the WebDriver protocol
  itself.


## GitHub

GitHub is used both for issue tracking and test submissions; we
provide [a limited introduction][github-intro] to both git and
GitHub.

Pull Requests are automatically labeled based on the directory the
files they change are in; there are also comments added automatically
to notify a number of people: this list of people comes from OWNERS
files in those same directories and their parents (i.e., they work
recursively: `a/OWNERS` will get notified for `a/foo.html` and
`a/b/bar.html`).

If you want to be notified about changes to tests in a directory, feel
free to add yourself to the OWNERS file: there's no requirement to own
anything as a result!


## Local Setup

The tests are designed to be run from your local computer. The test
environment requires [Python 2.7+](http://www.python.org/downloads) (but not Python 3.x).
You will also need a copy of OpenSSL.

On Windows, be sure to add the Python directory (`c:\python2x`, by default) to
your `%Path%` [Environment Variable](http://www.computerhope.com/issues/ch000549.htm),
and read the [Windows Notes](#windows-notes) section below.

To get the tests running, you need to set up the test domains in your
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

If you are behind a proxy, you also need to make sure the domains above are
excluded from your proxy lookups.

Because web-platform-tests uses git submodules, you must ensure that
these are up to date. In the root of your checkout, run:

```
git submodule update --init --recursive
```

The test environment can then be started using

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
little extra setup, i.e.:

Run the installer for Win32_OpenSSL_v1.1.0b (30MB). During installation,
change the default location for where to Copy OpenSSL Dlls from the
System directory to the /bin directory.

After installation, ensure that the path to OpenSSL (typically,
this will be `C:\OpenSSL-Win32\bin`) is in your `%Path%`
[Environment Variable](http://www.computerhope.com/issues/ch000549.htm).
If you forget to do this part, you will most likely see a 'File Not Found'
error when you start wptserve.

Finally, set the path value in the server configuration file to the
default OpenSSL configuration file location. To do this,
copy `config.default.json` in the web-platform-tests root to `config.json`.
Then edit the JSON so that the key `ssl/openssl/base_conf_path` has a
value that is the path to the OpenSSL config file (typically this
will be `C:\\OpenSSL-Win32\\bin\\openssl.cfg`).

Alternatively, you may also use
[Bash on Ubuntu on Windows](https://msdn.microsoft.com/en-us/commandline/wsl/about)
in the Windows 10 Anniversary Update build, then access your windows
partition from there to launch wptserve.


[web-platform]: https://platform.html5.org
[test262]: https://github.com/tc39/test262
[webgl]: https://github.com/KhronosGroup/WebGL
[public-test-infra]: https://lists.w3.org/Archives/Public/public-test-infra/
[IRC]: irc://irc.w3.org:6667/testing
[web irc]: http://irc.w3.org

[reftests]: {{ site.baseurl }}{% link _writing-tests/reftests.md %}
[testharness.js]: {{ site.baseurl }}{% link _writing-tests/testharness.md %}
[visual]: {{ site.baseurl }}{% link _writing-tests/visual.md %}
[manual]: {{ site.baseurl }}{% link _writing-tests/manual.md %}
[github-intro]: {{ site.baseurl }}{% link _appendix/github-intro.md %}
