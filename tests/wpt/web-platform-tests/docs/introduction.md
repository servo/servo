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
single specification, with the exception of `css/` which contains
testsuites for CSS WG specifications. For W3C specs, these directories
are typically named after the shortname of the spec (i.e. the name
used for snapshot publications under `/TR/`); for WHATWG specs, they
are typically named after the subdomain of the spec (i.e. trimming
`.spec.whatwg.org` from the URL); for other specs, something deemed
sensible is used. In any case, there are occasional exceptions for
historic reasons.

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

[GitHub](https://github.com/web-platform-tests/wpt/) is used both for [issue tracking](https://github.com/web-platform-tests/wpt/issues) and [test submissions](https://github.com/web-platform-tests/wpt/pulls); we
provide [a limited introduction][github-intro] to both git and
GitHub.

Pull Requests are automatically labeled based on the directory the
files they change are in; there are also comments added automatically
to notify a number of people: this list of people comes from META.yml
files in those same directories and their parents (i.e., they work
recursively: `a/META.yml` will get notified for `a/foo.html` and
`a/b/bar.html`).

If you want to be notified about changes to tests in a directory, feel
free to add yourself to the META.yml file!


## Local Setup

The tests are designed to be run from your local computer. The test
environment requires [Python 2.7+](http://www.python.org/downloads) (but not Python 3.x).

On Windows, be sure to add the Python directory (`c:\python2x`, by default) to
your `%Path%` [Environment Variable](http://www.computerhope.com/issues/ch000549.htm),
and read the [Windows Notes](#windows-notes) section below.

To get the tests running, you need to set up the test domains in your
[`hosts` file](http://en.wikipedia.org/wiki/Hosts_%28file%29%23Location_in_the_file_system).

The necessary content can be generated with `./wpt make-hosts-file`; on
Windows, you will need to preceed the prior command with `python` or
the path to the Python binary (`python wpt make-hosts-file`).

For example, on most UNIX-like systems, you can setup the hosts file with:

```bash
./wpt make-hosts-file | sudo tee -a /etc/hosts
```

And on Windows (this must be run in a PowerShell session with Administrator privileges):

```bash
python wpt make-hosts-file | Out-File %SystemRoot%\System32\drivers\etc\hosts -Encoding ascii -Append
```

If you are behind a proxy, you also need to make sure the domains above are
excluded from your proxy lookups.

The test environment can then be started using

    ./wpt serve

This will start HTTP servers on two ports and a websockets server on
one port. By default the web servers start on ports 8000 and 8443 and the other
ports are randomly-chosen free ports. Tests must be loaded from the
*first* HTTP server in the output. To change the ports,
create a `config.json` file in the wpt root directory, and add
port definitions of your choice e.g.:

```
{
  "ports": {
    "http": [1234, "auto"],
    "https":[5678]
  }
}
```

After your `hosts` file is configured, the servers will be locally accessible at:

http://web-platform.test:8000/<br>
https://web-platform.test:8443/ *

\**See [Trusting Root CA](https://github.com/web-platform-tests/wpt/blob/master/README.md#trusting-root-ca)*

## Running tests automatically

The `wpt run` command provides a frontend for running tests automatically
in various browsers. The general syntax is:

```
wpt run [options] <product> [test paths]
```

e.g. to run `dom/historical.html` in Firefox, the required command is:

```
wpt run firefox dom/historical.html
```

### Windows Notes

Generally Windows Subsystem for Linux will provide the smoothest user
experience for running web-platform-tests on Windows.

The standard Windows shell requires that all `wpt` commands are prefixed
by the Python binary i.e. assuming `python` is on your path the server is
started using:

`python wpt serve`


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
