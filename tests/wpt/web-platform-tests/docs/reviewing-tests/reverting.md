# Reverting Changes

Testing is imperfect and from time to time changes are merged into master which
break things for users of web-platform-tests. Such breakage can include:

  * Failures in Travis or Taskcluster runs for this repository, either on the
    master branch or on pull requests following the breaking change.

  * Breakage in browser engine repositories which import and run
    web-platform-tests, such as Chromium, Edge, Gecko, Servo and WebKit.

  * Breakage in results collections systems for results dashboards, such as
    [wpt.fyi](https://wpt.fyi).

  * Breakage in supplemental tooling used by working groups, such as the
    [CSS build system][].

When such breakage happens, if the maintainers of the affected systems request
it, pull requests to revert the original change should normally be approved and
merged as soon as possible. (When the original change itself was fixing a
serious problem, it's a judgement call, but prefer the fastest path to a stable
state acceptable to everyone.)

Once a revert has happened, the maintainers of the affected systems are
expected to work with the original patch author to resolve the problem so that
the change can be relanded. A reasonable timeframe to do so is within one week.

[CSS build system]: https://github.com/web-platform-tests/wpt/tree/master/css/tools
