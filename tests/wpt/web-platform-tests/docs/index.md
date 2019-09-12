# web-platform-tests documentation

The web-platform-tests project is a W3C-coordinated attempt to build a
cross-browser test suite for [the Web-platform
stack](https://platform.html5.org). Writing tests in a way that allows them to
be run in all browsers gives browser projects confidence that they are shipping
software which is compatible with other implementations, and that later
implementations will be compatible with their implementations. This in turn
gives Web authors/developers confidence that they can actually rely on the Web
platform to deliver on the promise of working across browsers and devices
without needing extra layers of abstraction to paper over the gaps left by
specification editors and implementors.

The most important sources of information and activity are:

- [github.com/web-platform-tests/wpt](https://github.com/web-platform-tests/wpt):
  the canonical location of the project's source code revision history and the
  discussion forum for changes to the code
- [web-platform-tests.org](https://web-platform-tests.org): the documentation
  website; details how to set up the project, how to write tests, how to give
  and receive peer review, how to serve as an administrator, and more
- [web-platform-tests.live](http://web-platform-tests.live): a public
  deployment of the test suite, allowing anyone to run the tests by visiting
  from an Internet-enabled browser of their choice
- [wpt.fyi](https://wpt.fyi): an archive of test results collected from an
  array of web browsers on a regular basis
- [Real-time chat room](http://irc.w3.org/?channels=testing): the
  [IRC](http://www.irchelp.org/) chat room named `#testing` on
  [irc.w3.org](https://www.w3.org/wiki/IRC); includes participants located
  around the world, but busiest during the European working day; [all
  discussion is archived here](https://w3.logbot.info/testing)
- [Mailing list](https://lists.w3.org/Archives/Public/public-test-infra/): a
  public and low-traffic discussion list

**If you'd like clarification about anything**, don't hesitate to ask in the
chat room or on the mailing list.

## Watch a Talk

If you prefer watching a video, here is a talk introducing web-platform-tests:

<iframe width="560" height="315" src="https://www.youtube.com/embed/XnfE3MfH5hQ" frameborder="0" allow="autoplay; encrypted-media" allowfullscreen></iframe>

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

## Table of Contents

```eval_rst
.. toctree::
   :maxdepth: 2

   test-suite-design
   running-tests/index
   writing-tests/index
   reviewing-tests/index
   admin/index
```

[github-intro]: writing-tests/github-intro
