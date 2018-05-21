This directory contains tests for the
[Async Cookies API](https://github.com/WICG/cookie-store).

## Note on cookie naming conventions

A simple origin cookie is a cookie named with the `__Host-` prefix
which is always secure-flagged, always implicit-domain, always
`/`-scoped, and hence always unambiguous in the cookie jar serialization
and origin-scoped. It can be treated as a simple key/value pair.

`"LEGACY"` in a cookie name here means it is an old-style unprefixed
cookie name, so you can't tell e.g. whether it is Secure-flagged or
`/`-pathed just by looking at it, and its flags, domain and path may
vary even in a single cookie jar serialization leading to apparent
duplicate entries, ambiguities, and complexity (i.e. it cannot be
treated as a simple key/value pair.)

Cookie names used in the tests are intended to be
realistic. Traditional session cookie names are typically
all-upper-case for broad framework compatibility. The more modern
`"__Host-"` prefix has only one allowed casing. An expected upgrade
path from traditional "legacy" cookie names to simple origin cookie
names is simply to prefix the traditional name with the `"__Host-"`
prefix.

Many of the used cookie names are non-ASCII to ensure
straightforward internationalization is possible at every API surface.
These work in many modern browsers, though not yet all of them.
