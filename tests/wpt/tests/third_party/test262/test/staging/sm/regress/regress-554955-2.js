/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
function f(s) {
    eval(s);
    return function(a) {
        with({}) {}; // repel JägerMonkey
        eval(a);
        return b;
    };
}

var b = 1;
var g1 = f("");
var g2 = f("var b = 2;");

/* Call the lambda once, caching a reference to the global b. */
g1('');

/*
 * If this call sees the above cache entry, then it will erroneously use
 * the global b.
 */
assert.sameValue(g2(''), 2);

