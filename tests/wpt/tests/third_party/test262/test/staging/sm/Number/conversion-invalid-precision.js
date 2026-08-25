/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Number.prototype.to* should throw a RangeError when passed a bad precision
info: bugzilla.mozilla.org/show_bug.cgi?id=795745
esid: pending
---*/

function test(method, prec) {
  assert.throws(RangeError, function() {
    Number.prototype[method].call(0, prec);
  });
}

test("toExponential", -32);
test("toFixed", -32);
test("toPrecision", -32);

test("toExponential", 9999999);
test("toFixed", 9999999);
test("toPrecision", 9999999);

test("toPrecision", 0);
