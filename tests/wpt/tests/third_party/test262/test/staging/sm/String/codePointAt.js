// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  String.prototype.codePointAt
info: bugzilla.mozilla.org/show_bug.cgi?id=918879
esid: pending
---*/

// Tests taken from:
// https://github.com/mathiasbynens/String.prototype.codePointAt/blob/master/tests/tests.js
assert.sameValue(String.prototype.codePointAt.length, 1);
assert.sameValue(String.prototype.propertyIsEnumerable('codePointAt'), false);

// String that starts with a BMP symbol
assert.sameValue('abc\uD834\uDF06def'.codePointAt(''), 0x61);
assert.sameValue('abc\uD834\uDF06def'.codePointAt('_'), 0x61);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(), 0x61);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(-Infinity), undefined);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(-1), undefined);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(-0), 0x61);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(0), 0x61);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(3), 0x1D306);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(4), 0xDF06);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(5), 0x64);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(42), undefined);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(Infinity), undefined);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(Infinity), undefined);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(NaN), 0x61);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(false), 0x61);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(null), 0x61);
assert.sameValue('abc\uD834\uDF06def'.codePointAt(undefined), 0x61);

// String that starts with an astral symbol
assert.sameValue('\uD834\uDF06def'.codePointAt(''), 0x1D306);
assert.sameValue('\uD834\uDF06def'.codePointAt('1'), 0xDF06);
assert.sameValue('\uD834\uDF06def'.codePointAt('_'), 0x1D306);
assert.sameValue('\uD834\uDF06def'.codePointAt(), 0x1D306);
assert.sameValue('\uD834\uDF06def'.codePointAt(-1), undefined);
assert.sameValue('\uD834\uDF06def'.codePointAt(-0), 0x1D306);
assert.sameValue('\uD834\uDF06def'.codePointAt(0), 0x1D306);
assert.sameValue('\uD834\uDF06def'.codePointAt(1), 0xDF06);
assert.sameValue('\uD834\uDF06def'.codePointAt(42), undefined);
assert.sameValue('\uD834\uDF06def'.codePointAt(false), 0x1D306);
assert.sameValue('\uD834\uDF06def'.codePointAt(null), 0x1D306);
assert.sameValue('\uD834\uDF06def'.codePointAt(undefined), 0x1D306);

// Lone high surrogates
assert.sameValue('\uD834abc'.codePointAt(''), 0xD834);
assert.sameValue('\uD834abc'.codePointAt('_'), 0xD834);
assert.sameValue('\uD834abc'.codePointAt(), 0xD834);
assert.sameValue('\uD834abc'.codePointAt(-1), undefined);
assert.sameValue('\uD834abc'.codePointAt(-0), 0xD834);
assert.sameValue('\uD834abc'.codePointAt(0), 0xD834);
assert.sameValue('\uD834abc'.codePointAt(false), 0xD834);
assert.sameValue('\uD834abc'.codePointAt(NaN), 0xD834);
assert.sameValue('\uD834abc'.codePointAt(null), 0xD834);
assert.sameValue('\uD834abc'.codePointAt(undefined), 0xD834);

// Lone low surrogates
assert.sameValue('\uDF06abc'.codePointAt(''), 0xDF06);
assert.sameValue('\uDF06abc'.codePointAt('_'), 0xDF06);
assert.sameValue('\uDF06abc'.codePointAt(), 0xDF06);
assert.sameValue('\uDF06abc'.codePointAt(-1), undefined);
assert.sameValue('\uDF06abc'.codePointAt(-0), 0xDF06);
assert.sameValue('\uDF06abc'.codePointAt(0), 0xDF06);
assert.sameValue('\uDF06abc'.codePointAt(false), 0xDF06);
assert.sameValue('\uDF06abc'.codePointAt(NaN), 0xDF06);
assert.sameValue('\uDF06abc'.codePointAt(null), 0xDF06);
assert.sameValue('\uDF06abc'.codePointAt(undefined), 0xDF06);

(function() { String.prototype.codePointAt.call(undefined); }, TypeError);
assert.throws(TypeError, function() { String.prototype.codePointAt.call(undefined, 4); });
assert.throws(TypeError, function() { String.prototype.codePointAt.call(null); });
assert.throws(TypeError, function() { String.prototype.codePointAt.call(null, 4); });
assert.sameValue(String.prototype.codePointAt.call(42, 0), 0x34);
assert.sameValue(String.prototype.codePointAt.call(42, 1), 0x32);
assert.sameValue(String.prototype.codePointAt.call({ 'toString': function() { return 'abc'; } }, 2), 0x63);

assert.throws(TypeError, function() { String.prototype.codePointAt.apply(undefined); });
assert.throws(TypeError, function() { String.prototype.codePointAt.apply(undefined, [4]); });
assert.throws(TypeError, function() { String.prototype.codePointAt.apply(null); });
assert.throws(TypeError, function() { String.prototype.codePointAt.apply(null, [4]); });
assert.sameValue(String.prototype.codePointAt.apply(42, [0]), 0x34);
assert.sameValue(String.prototype.codePointAt.apply(42, [1]), 0x32);
assert.sameValue(String.prototype.codePointAt.apply({ 'toString': function() { return 'abc'; } }, [2]), 0x63);
