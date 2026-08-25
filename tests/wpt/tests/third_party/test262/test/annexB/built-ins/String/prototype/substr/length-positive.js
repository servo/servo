// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.substr
es6id: B.2.3.1
description: Behavior when "length" is a positive number
info: |
    [...]
    4. If length is undefined, let end be +∞; otherwise let end be ?
       ToInteger(length).
    [...]
    7. Let resultLength be min(max(end, 0), size - intStart).
    8. If resultLength ≤ 0, return the empty String "".
    9. Return a String containing resultLength consecutive code units from S
       beginning with the code unit at index intStart.
---*/

assert.sameValue('abc'.substr(0, 1), 'a', '0, 1');
assert.sameValue('abc'.substr(0, 2), 'ab', '0, 1');
assert.sameValue('abc'.substr(0, 3), 'abc', '0, 1');
assert.sameValue('abc'.substr(0, 4), 'abc', '0, 1');

assert.sameValue('abc'.substr(1, 1), 'b', '1, 1');
assert.sameValue('abc'.substr(1, 2), 'bc', '1, 1');
assert.sameValue('abc'.substr(1, 3), 'bc', '1, 1');
assert.sameValue('abc'.substr(1, 4), 'bc', '1, 1');

assert.sameValue('abc'.substr(2, 1), 'c', '2, 1');
assert.sameValue('abc'.substr(2, 2), 'c', '2, 1');
assert.sameValue('abc'.substr(2, 3), 'c', '2, 1');
assert.sameValue('abc'.substr(2, 4), 'c', '2, 1');

assert.sameValue('abc'.substr(3, 1), '', '3, 1');
assert.sameValue('abc'.substr(3, 2), '', '3, 1');
assert.sameValue('abc'.substr(3, 3), '', '3, 1');
assert.sameValue('abc'.substr(3, 4), '', '3, 1');
