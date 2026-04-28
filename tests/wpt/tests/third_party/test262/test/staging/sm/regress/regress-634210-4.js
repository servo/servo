/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
function outer() {
    function f() {}
    f.p = function() {};
    Object.seal(f);
    return f.p;
}

var m1 = outer();
var m2 = outer();
assert.sameValue(m1 === m2, false);

m1.foo = "hi";
assert.sameValue(m2.foo, undefined);

