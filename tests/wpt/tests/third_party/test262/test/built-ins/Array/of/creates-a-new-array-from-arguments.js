// Copyright (c) 2015 the V8 project authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
/*---
esid: sec-array.of
es6id: 22.1.2.3
description: >
  Array.of method creates a new Array with a variable number of arguments.
info: |
  22.1.2.3 Array.of ( ...items )

  ...
  7. Let k be 0.
  8. Repeat, while k < len
    a. Let kValue be items[k].
    b. Let Pk be ToString(k).
    c. Let defineStatus be CreateDataPropertyOrThrow(A,Pk, kValue).
    d. ReturnIfAbrupt(defineStatus).
    e. Increase k by 1.
  9. Let setStatus be Set(A, "length", len, true).
  10. ReturnIfAbrupt(setStatus).
  11. Return A.
---*/

var a1 = Array.of('Mike', 'Rick', 'Leo');
assert.sameValue(
  a1.length, 3,
  'The value of a1.length is expected to be 3'
);
assert.sameValue(a1[0], 'Mike', 'The value of a1[0] is expected to be "Mike"');
assert.sameValue(a1[1], 'Rick', 'The value of a1[1] is expected to be "Rick"');
assert.sameValue(a1[2], 'Leo', 'The value of a1[2] is expected to be "Leo"');

var a2 = Array.of(undefined, false, null, undefined);
assert.sameValue(
  a2.length, 4,
  'The value of a2.length is expected to be 4'
);
assert.sameValue(a2[0], undefined, 'The value of a2[0] is expected to equal undefined');
assert.sameValue(a2[1], false, 'The value of a2[1] is expected to be false');
assert.sameValue(a2[2], null, 'The value of a2[2] is expected to be null');
assert.sameValue(a2[3], undefined, 'The value of a2[3] is expected to equal undefined');

var a3 = Array.of();
assert.sameValue(a3.length, 0, 'The value of a3.length is expected to be 0');
