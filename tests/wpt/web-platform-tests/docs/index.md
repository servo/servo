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

## Help!

If you get stuck or want clarification about anything, feel free to
ask on either the [mailing list][public-test-infra] or [IRC][]
([webclient][web irc], join channel `#testing`); IRC is generally
busiest during the European working day but frequently has people on
it at all times and should probably be the general first port of call
for any help.

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

[public-test-infra]: https://lists.w3.org/Archives/Public/public-test-infra/
[IRC]: irc://irc.w3.org:6667/testing
[web irc]: http://irc.w3.org
[github-intro]: writing-tests/github-intro
