/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/
var obj = {};
var sym = Symbol();

var gets = 0;
var sets = [];
Object.defineProperty(obj, sym, {
    get: function () { return ++gets; },
    set: function (v) { sets.push(v); }
});

// getter
for (var i = 1; i < 9; i++)
    assert.sameValue(obj[sym], i);

// setter
var expected = [];
for (var i = 0; i < 9; i++) {
    assert.sameValue(obj[sym] = i, i);
    expected.push(i);
}
assert.compareArray(sets, expected);

// increment operator
gets = 0;
sets = [];
assert.sameValue(obj[sym]++, 1);
assert.compareArray(sets, [2]);

// assignment
gets = 0;
sets = [];
assert.sameValue(obj[sym] *= 12, 12);
assert.compareArray(sets, [12]);
