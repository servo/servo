/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Array.prototype.unshift without args
info: bugzilla.mozilla.org/show_bug.cgi?id=614070
esid: pending
---*/

// ES6 ToLength clamps length values to 2^53 - 1.
var MAX_LENGTH = 2**53 - 1;

var a = {};
a.length = MAX_LENGTH + 1;
assert.sameValue([].unshift.call(a), MAX_LENGTH);
assert.sameValue(a.length, MAX_LENGTH);

function testGetSet(len, expected) {
    var newlen;
    var a = { get length() { return len; }, set length(v) { newlen = v; } };
    var res = [].unshift.call(a);
    assert.sameValue(res, expected);
    assert.sameValue(newlen, expected);
}

testGetSet(0, 0);
testGetSet(10, 10);
testGetSet("1", 1);
testGetSet(null, 0);
testGetSet(MAX_LENGTH + 2, MAX_LENGTH);
testGetSet(-5, 0);
