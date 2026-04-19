/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Scripted proxies' [[OwnPropertyKeys]] should have linear complexity
info: bugzilla.mozilla.org/show_bug.cgi?id=1257779
esid: pending
---*/

// Making this 50k makes cgc builds time out on tbpl.  5k takes 28s locally.
// 10k takes 84s locally.  So pick an intermediate number, with a generous
// constant factor in case cgc-on-tbpl is much slower.
const HALF_COUNT = 7500;

var configurables = [];
for (var i = 0; i < HALF_COUNT; i++)
  configurables.push("conf" + i);

var nonconfigurables = [];
for (var i = 0; i < HALF_COUNT; i++)
  nonconfigurables.push("nonconf" + i);

var target = {};
for (var name of configurables)
  Object.defineProperty(target, name, { configurable: false, value: 0 });
for (var name of nonconfigurables)
  Object.defineProperty(target, name, { configurable: true, value:  0 });

var handler = {
  ownKeys(t) {
    assert.sameValue(t, target, "target mismatch!");

    var trapResult = [];

    // Append all nonconfigurables in reverse order of presence.
    for (var i = nonconfigurables.length; i > 0; i--)
      trapResult.push(nonconfigurables[i - 1]);

    // Then the same for all configurables.
    for (var i = configurables.length; i > 0; i--)
      trapResult.push(configurables[i - 1]);

    // The end consequence is that this ordering is exactly opposite the
    // ordering they'll have on the target, and so worst-case performance will
    // occur if the spec's |uncheckedResultKeys| structure is a vector having
    // the same order as |trapResult|, searched from beginning to end in the
    // presence-checks in the last few steps of the [[OwnPropertyKeys]]
    // algorithm.
    return trapResult;
  }
};

var p = new Proxy(target, handler);

// The test passes if it doesn't time out.
assert.sameValue(Object.getOwnPropertyNames(p).length, HALF_COUNT * 2);
