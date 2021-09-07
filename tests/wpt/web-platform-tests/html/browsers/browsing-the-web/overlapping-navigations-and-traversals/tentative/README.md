# Overlapping navigation and traversal tests

While developing [app history](https://wicg.github.io/app-history/), @domenic
noticed that cancelation of navigations and history traversals is not very
well-defined in the spec.

On the spec side, this will probably be fixed as part of, or after, the
[session history rewrite](https://github.com/whatwg/html/pull/6315).

In the meantime, this directory contains tests which try to match most browsers,
or picks one of the potential behaviors.

<https://github.com/whatwg/html/issues/6927> discusses these results.
