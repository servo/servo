The web-platform-tests Project [![IRC chat](https://goo.gl/6nCIks)](http://irc.w3.org/?channels=testing)
==============================

The web-platform-tests Project is a W3C-coordinated attempt to build a
cross-browser test suite for the Web-platform stack. Writing tests in a way
that allows them to be run in all browsers gives browser projects
confidence that they are shipping software that is compatible with other
implementations, and that later implementations will be compatible with
their implementations. This in turn gives Web authors/developers
confidence that they can actually rely on the Web platform to deliver on
the promise of working across browsers and devices without needing extra
layers of abstraction to paper over the gaps left by specification
editors and implementors.

Setting Up the Repo
===================

Clone or otherwise get https://github.com/web-platform-tests/wpt.

Note: because of the frequent creation and deletion of branches in this
repo, it is recommended to "prune" stale branches when fetching updates,
i.e. use `git pull --prune` (or `git fetch -p && git merge`).

Running the Tests
=================

The tests are designed to be run from your local computer. The test
environment requires [Python 2.7+](http://www.python.org/downloads) (but not Python 3.x).

On Windows, be sure to add the Python directory (`c:\python2x`, by default) to
your `%Path%` [Environment Variable](http://www.computerhope.com/issues/ch000549.htm),
and read the [Windows Notes](#windows-notes) section below.

To get the tests running, you need to set up the test domains in your
[`hosts` file](http://en.wikipedia.org/wiki/Hosts_%28file%29%23Location_in_the_file_system).

The necessary content can be generated with `./wpt make-hosts-file`; on
Windows, you will need to precede the prior command with `python` or
the path to the Python binary (`python wpt make-hosts-file`).

For example, on most UNIX-like systems, you can setup the hosts file with:

```bash
./wpt make-hosts-file | sudo tee -a /etc/hosts
```

And on Windows (this must be run in a PowerShell session with Administrator privileges):

```powershell
python wpt make-hosts-file | Out-File $env:systemroot\System32\drivers\etc\hosts -Encoding ascii -Append
```

If you are behind a proxy, you also need to make sure the domains above are
excluded from your proxy lookups.


Running Tests Manually
======================

The test server can be started using
```
./wpt serve
```

**On Windows**: You will need to precede the prior command with
`python` or the path to the python binary.
```bash
python wpt serve
```

This will start HTTP servers on two ports and a websockets server on
one port. By default the web servers start on ports 8000 and 8443 and
the other ports are randomly-chosen free ports. Tests must be loaded
from the *first* HTTP server in the output. To change the ports,
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

To use the web-based runner point your browser to:

http://web-platform.test:8000/tools/runner/index.html <br>
https://web-platform.test:8443/tools/runner/index.html *

\**See [Trusting Root CA](#trusting-root-ca)*

Running Tests Automatically
---------------------------

Tests can be run automatically in a browser using the `run` command of
the `wpt` script in the root of the checkout. This requires the hosts
file setup documented above, but you must *not* have the
test server already running when calling `wpt run`. The basic command
line syntax is:

```bash
./wpt run product [tests]
```

**On Windows**: You will need to precede the prior command with
`python` or the path to the python binary.
```bash
python wpt run product [tests]
```

where `product` is currently `firefox` or `chrome` and `[tests]` is a
list of paths to tests. This will attempt to automatically locate a
browser instance and install required dependencies. The command is
very configurable; for example to specify a particular binary use
`wpt run --binary=path product`. The full range of options can be see
with `wpt run --help` and `wpt run --wptrunner-help`.

Not all dependencies can be automatically installed; in particular the
`certutil` tool required to run https tests with Firefox must be
installed using a system package manager or similar.

On Debian/Ubuntu certutil may be installed using:

```
sudo apt install libnss3-tools
```

And on macOS with homebrew using:

```
brew install nss
```

On other platforms, download the firefox archive and common.tests.tar.gz
archive for your platform from
[Mozilla CI](https://archive.mozilla.org/pub/firefox/nightly/latest-mozilla-central/).

Then extract `certutil[.exe]` from the tests.tar.gz package and
`libnss3[.so|.dll|.dynlib]` and put the former on your path and the latter on
your library path.


Command Line Tools
==================

The `wpt` command provides a frontend to a variety of tools for
working with and running web-platform-tests. Some of the most useful
commands are:

* `wpt serve` - For starting the wpt http server
* `wpt run` - For running tests in a browser
* `wpt lint` - For running the lint against all tests
* `wpt manifest` - For updating or generating a `MANIFEST.json` test manifest
* `wpt install` - For installing the latest release of a browser or
  webdriver server on the local machine.

<span id="windows-notes">Windows Notes</span>
=============================================

On Windows `wpt` commands must be prefixed with `python` or the path
to the python binary (if `python` is not in your `%PATH%`).

```bash
python wpt [command]
```

Alternatively, you may also use
[Bash on Ubuntu on Windows](https://msdn.microsoft.com/en-us/commandline/wsl/about)
in the Windows 10 Anniversary Update build, then access your windows
partition from there to launch `wpt` commands.

Please make sure git and your text editor do not automatically convert
line endings, as it will cause lint errors. For git, please set
`git config core.autocrlf false` in your working tree.

Publication
===========

The master branch is automatically synced to http://w3c-test.org/.

Pull requests are
[automatically mirrored](http://w3c-test.org/submissions/) except those
that modify sensitive resources (such as `.py`). The latter require
someone with merge access to comment with "LGTM" or "w3c-test:mirror" to
indicate the pull request has been checked.

Branches
========

In the vast majority of cases the **only** upstream branch that you
should need to care about is `master`. If you see other branches in
the repository, you can generally safely ignore them.

Contributing
============

Save the Web, Write Some Tests!

Absolutely everyone is welcome (and even encouraged) to contribute to
test development, so long as you fulfill the contribution requirements
detailed in the [Contributing Guidelines][contributing]. No test is
too small or too simple, especially if it corresponds to something for
which you've noted an interoperability bug in a browser.

The way to contribute is just as usual:

* Fork this repository (and make sure you're still relatively in sync
  with it if you forked a while ago).
* Create a branch for your changes:
  `git checkout -b topic`.
* Make your changes.
* Run the lint script described below.
* Commit locally and push that to your repo.
* Send in a pull request based on the above.

Issues with web-platform-tests
------------------------------

If you spot an issue with a test and are not comfortable providing a
pull request per above to fix it, please
[file a new issue](https://github.com/web-platform-tests/wpt/issues/new).
Thank you!

Lint tool
---------

We have a lint tool for catching common mistakes in test files. You
can run it manually by starting the `lint` executable from the root of
your local web-platform-tests working directory like this:

```
./wpt lint
```

The lint tool is also run automatically for every submitted pull
request, and reviewers will not merge branches with tests that have
lint errors, so you must fix any errors the lint tool reports.

In the unusual case of error reports for things essential to a
certain test or that for other exceptional reasons shouldn't prevent
a merge of a test, update and commit the `lint.whitelist` file in the
web-platform-tests root directory to suppress the error reports.

For more details, see the [lint-tool documentation][lint-tool].

[lint-tool]: https://web-platform-tests.org/writing-tests/lint-tool.html

Getting Involved
================

If you wish to contribute actively, you're very welcome to join the
public-test-infra@w3.org mailing list (low traffic) by
[signing up to our mailing list](mailto:public-test-infra-request@w3.org?subject=subscribe).
The mailing list is [archived][mailarchive].

Join us on irc #testing ([irc.w3.org][ircw3org], port 6665). The channel
is [archived][ircarchive].

[contributing]: https://github.com/web-platform-tests/wpt/blob/master/CONTRIBUTING.md
[ircw3org]: https://www.w3.org/wiki/IRC
[ircarchive]: https://w3.logbot.info/testing
[mailarchive]: https://lists.w3.org/Archives/Public/public-test-infra/

Documentation
=============

* [How to write and review tests](https://web-platform-tests.org/)
* [Documentation for the wptserve server](http://wptserve.readthedocs.org/en/latest/)
