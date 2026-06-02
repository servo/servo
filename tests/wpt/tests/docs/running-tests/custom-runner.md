# Writing Your Own Runner

Most test runners have two stages: finding all tests, followed by
executing them (or a subset thereof).

To find all tests in the repository, it is **strongly** recommended to
use the included `wpt manifest` tool: the required behaviors are more
complex than what are documented (especially when it comes to
precedence of the various possibilities and some undocumented legacy
ways to define test types), and hence its behavior should be
considered the canonical definition of how to enumerate tests and find
their type in the repository.

For test execution, please read the documentation for the various test
types very carefully and then check your understanding on the [mailing
list][public-test-infra] or [matrix][matrix]. It's possible edge-case
behavior isn't properly documented!

[public-test-infra]: https://lists.w3.org/Archives/Public/public-test-infra/
[matrix]: https://app.element.io/#/room/#wpt:matrix.org
[web irc]: http://irc.w3.org
