/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
// Shu's test
function test(makeNonArray) {
    function C() {}
    C.prototype = []
    if (makeNonArray)
        C.prototype.constructor = C
    var c = new C();
    c.push("foo");
    return c.length
}
assert.sameValue(test(true), 1);
assert.sameValue(test(false), 1);

// jorendorff's longer test
var a = [];
a.slowify = 1;
var b = Object.create(a);
b.length = 12;
assert.sameValue(b.length, 12);

// jorendorff's shorter test
var b = Object.create(Array.prototype);
b.length = 12;
assert.sameValue(b.length, 12);

