The web-platform-tests Project
==============================

[![Taskcluster CI Status](https://community-tc.services.mozilla.com/api/github/v1/repository/web-platform-tests/wpt/master/badge.svg)](https://community-tc.services.mozilla.com/api/github/v1/repository/web-platform-tests/wpt/master/latest) [![documentation](https://github.com/web-platform-tests/wpt/workflows/documentation/badge.svg)](https://github.com/web-platform-tests/wpt/actions?query=workflow%3Adocumentation+branch%3Amaster) [![manifest](https://github.com/web-platform-tests/wpt/workflows/manifest/badge.svg)](https://github.com/web-platform-tests/wpt/actions?query=workflow%3Amanifest+branch%3Amaster) [![Python 3](https://pyup.io/repos/github/web-platform-tests/wpt/python-3-shield.svg)](https://pyup.io/repos/github/web-platform-tests/wpt/)

The web-platform-tests Project is a cross-browser test suite for the
Web-platform stack. Writing tests in a way that allows them to be run in all
browsers gives browser projects confidence that they are shipping software that
is compatible with other implementations, and that later implementations will
be compatible with their implementations. This in turn gives Web
authors/developers confidence that they can actually rely on the Web platform
to deliver on the promise of working across browsers and devices without
needing extra layers of abstraction to paper over the gaps left by
specification editors and implementors.

The most important sources of information and activity are:

- [github.com/web-platform-tests/wpt](https://github.com/web-platform-tests/wpt):
  the canonical location of the project's source code revision history and the
  discussion forum for changes to the code
- [web-platform-tests.org](https://web-platform-tests.org): the documentation
  website; details how to set up the project, how to write tests, how to give
  and receive peer review, how to serve as an administrator, and more
- [wpt.live](https://wpt.live): a public deployment of the test suite,
  allowing anyone to run the tests by visiting from an
  Internet-enabled browser of their choice
- [wpt.fyi](https://wpt.fyi): an archive of test results collected from an
  array of web browsers on a regular basis
- [Real-time chat room](https://app.element.io/#/room/#wpt:matrix.org): the
  `wpt:matrix.org` matrix channel; includes participants located
  around the world, but busiest during the European working day.
- [Mailing list](https://lists.w3.org/Archives/Public/public-test-infra/): a
  public and low-traffic discussion list
- [RFCs](https://github.com/web-platform-tests/rfcs): a repo for requesting
  comments on substantial changes that would impact other stakeholders or
  users; people who work on WPT infra are encouraged to watch the repo.

**If you'd like clarification about anything**, don't hesitate to ask in the
chat room or on the mailing list.

Setting Up the Repo
===================

Clone or otherwise get https://github.com/web-platform-tests/wpt.

Note: because of the frequent creation and deletion of branches in this
repo, it is recommended to "prune" stale branches when fetching updates,
i.e. use `git pull --prune` (or `git fetch -p && git merge`).

Running the Tests
=================

See the [documentation website](https://web-platform-tests.org/running-tests/)
and in particular the
[system setup for running tests locally](https://web-platform-tests.org/running-tests/from-local-system.html#system-setup).

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
* `wpt serve-wave` - For starting the wpt http server and the WAVE test runner.
For more details on how to use the WAVE test runner see the [documentation](./tools/wave/docs/usage/usage.md).

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

The master branch is automatically synced to [wpt.live](https://wpt.live/) and
[w3c-test.org](https://w3c-test.org/).

Contributing
============

Save the Web, Write Some Tests!

Absolutely everyone is welcome to contribute to test development. No
test is too small or too simple, especially if it corresponds to
something for which you've noted an interoperability bug in a browser.

The way to contribute is just as usual:

* Fork this repository (and make sure you're still relatively in sync
  with it if you forked a while ago).
* Create a branch for your changes:
  `git checkout -b topic`.
* Make your changes.
* Run `./wpt lint` as described above.
* Commit locally and push that to your repo.
* Create a pull request based on the above.

Issues with web-platform-tests
------------------------------

If you spot an issue with a test and are not comfortable providing a
pull request per above to fix it, please
[file a new issue](https://github.com/web-platform-tests/wpt/issues/new).
Thank you!
