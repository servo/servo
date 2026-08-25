// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.substr
es6id: B.2.3.1
description: Behavior when "length" is not defined
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

assert.sameValue('abc'.substr(0), 'abc', 'start: 0, length: unspecified');
assert.sameValue('abc'.substr(1), 'bc', 'start: 1, length: unspecified');
assert.sameValue('abc'.substr(2), 'c', 'start: 2, length: unspecified');
assert.sameValue('abc'.substr(3), '', 'start: 3, length: unspecified');

assert.sameValue(
  'abc'.substr(0, undefined), 'abc', 'start: 0, length: undefined'
);
assert.sameValue(
  'abc'.substr(1, undefined), 'bc', 'start: 1, length: undefined'
);
assert.sameValue(
  'abc'.substr(2, undefined), 'c', 'start: 2, length: undefined'
);
assert.sameValue(
  'abc'.substr(3, undefined), '', 'start: 3, length: undefined'
);
