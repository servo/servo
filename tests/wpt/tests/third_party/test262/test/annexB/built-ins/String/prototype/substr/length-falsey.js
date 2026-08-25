// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.substr
es6id: B.2.3.1
description: >
    Behavior when "length" is a false-converting value other than `undefined`
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

assert.sameValue('abc'.substr(0, false), '', 'start: 0, length: false');
assert.sameValue('abc'.substr(1, false), '', 'start: 1, length: false');
assert.sameValue('abc'.substr(2, false), '', 'start: 2, length: false');
assert.sameValue('abc'.substr(3, false), '', 'start: 3, length: false');

assert.sameValue('abc'.substr(0, NaN), '', 'start: 0, length: NaN');
assert.sameValue('abc'.substr(1, NaN), '', 'start: 1, length: NaN');
assert.sameValue('abc'.substr(2, NaN), '', 'start: 2, length: NaN');
assert.sameValue('abc'.substr(3, NaN), '', 'start: 3, length: NaN');

assert.sameValue('abc'.substr(0, ''), '', 'start: 0, length: ""');
assert.sameValue('abc'.substr(1, ''), '', 'start: 1, length: ""');
assert.sameValue('abc'.substr(2, ''), '', 'start: 2, length: ""');
assert.sameValue('abc'.substr(3, ''), '', 'start: 3, length: ""');

assert.sameValue('abc'.substr(0, null), '', 'start: 0, length: null');
assert.sameValue('abc'.substr(1, null), '', 'start: 1, length: null');
assert.sameValue('abc'.substr(2, null), '', 'start: 2, length: null');
assert.sameValue('abc'.substr(3, null), '', 'start: 3, length: null');
