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
    if (s) {
        function b() { }
    }
    return function(a) {
        eval(a);
        return b;
    };
}

var b = 1;
var g1 = f(false);
var g2 = f(true);

/* Call the lambda once, caching a reference to the global b. */
g1('');

/*
 * If this call sees the above cache entry, then it will erroneously use the
 * global b.
 */
assert.sameValue(typeof g2(''), "function");

