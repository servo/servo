// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.substr
es6id: B.2.3.1
description: Behavior when "length" is a negative number
info: |
    [...]
    7. Let resultLength be min(max(end, 0), size - intStart).
    8. If resultLength ≤ 0, return the empty String "".
---*/

assert.sameValue('abc'.substr(0, -1), '', '0, -1');
assert.sameValue('abc'.substr(0, -2), '', '0, -2');
assert.sameValue('abc'.substr(0, -3), '', '0, -3');
assert.sameValue('abc'.substr(0, -4), '', '0, -4');

assert.sameValue('abc'.substr(1, -1), '', '1, -1');
assert.sameValue('abc'.substr(1, -2), '', '1, -2');
assert.sameValue('abc'.substr(1, -3), '', '1, -3');
assert.sameValue('abc'.substr(1, -4), '', '1, -4');

assert.sameValue('abc'.substr(2, -1), '', '2, -1');
assert.sameValue('abc'.substr(2, -2), '', '2, -2');
assert.sameValue('abc'.substr(2, -3), '', '2, -3');
assert.sameValue('abc'.substr(2, -4), '', '2, -4');

assert.sameValue('abc'.substr(3, -1), '', '3, -1');
assert.sameValue('abc'.substr(3, -2), '', '3, -2');
assert.sameValue('abc'.substr(3, -3), '', '3, -3');
assert.sameValue('abc'.substr(3, -4), '', '3, -4');
