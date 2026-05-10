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
    var d;
    {
      let c = 3;
      d = function() { a; }; // force Block object to be cloned
      with({}) {}; // repel JägerMonkey
      return b; // lookup occurs in scope of Block
    }
  };
}

var b = 1;
var g1 = f("");
var g2 = f("var b = 2;");

/* Call the lambda once, caching a reference to the global b. */
g1(0);

/* 
 * If this call sees the above cache entry, then it will erroneously use the
 * global b.
 */
assert.sameValue(g2(0), 2);

