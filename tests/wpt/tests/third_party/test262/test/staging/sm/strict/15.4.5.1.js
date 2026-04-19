/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [compareArray.js]
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
var out = {};

function arr() {
  return Object.defineProperty([1, 2, 3, 4], 2, {configurable: false});
}

function nonStrict1(out)
{
  var a = out.array = arr();
  a.length = 2;
}

function strict1(out)
{
  "use strict";
  var a = out.array = arr();
  a.length = 2;
  return a;
}

out.array = null;
nonStrict1(out);
assert.compareArray(out.array, [1, 2, 3]);

out.array = null;

assert.throws(TypeError, function() {
  strict1(out);
});

assert.compareArray(out.array, [1, 2, 3]);
