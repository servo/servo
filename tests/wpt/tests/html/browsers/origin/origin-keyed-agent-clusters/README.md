# Origin-keyed agent clusters tests

These are tests for the [origin-keyed agent clusters](https://html.spec.whatwg.org/multipage/origin.html#origin-keyed-agent-clusters)
feature.

## Test filenames

The tests in `2-iframes` follow the file naming pattern

```
parent-[yes|no]-child1-[yes|no]-[designator]-child2-[yes|no]-[designator]
```

Here:

* `yes` or `no` refers to whether the `Origin-Agent-Cluster` header is set or
  unset.
* `designator` explains how the child differs from the parent: e.g. by being a
  subdomain, or having a different port, or both. There's also `same` if it's
  same-origin.

Other directories have variations on this, e.g. `1-iframe/` does the same thing
but for a single `child` instead of `child1` and `child2`, and `navigation/`
uses `1` and `2` to represent the two different locations the single iframe will
be navigated to.

## Coverage

Header parsing is covered by a few tests in the `1-iframe/` subdirectory, and
not duplicated to all other scenarios.
