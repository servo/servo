// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.substr
es6id: B.2.3.1
description: >
    Behavior when input string contains a surrogate pair
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

assert.sameValue('\ud834\udf06'.substr(0), '\ud834\udf06', 'start: 0');
assert.sameValue('\ud834\udf06'.substr(1), '\udf06', 'start: 1');
assert.sameValue('\ud834\udf06'.substr(2), '', 'start: 2');
assert.sameValue('\ud834\udf06'.substr(0, 0), '', 'end: 0');
assert.sameValue('\ud834\udf06'.substr(0, 1), '\ud834', 'end: 1');
assert.sameValue('\ud834\udf06'.substr(0, 2), '\ud834\udf06', 'end: 2');
