// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Spares arrays are only equivalent if they have the same length.
includes: [compareArray.js]
---*/


assert.compareArray([,], [,], 'Sparse arrays of the same length are equivalent.');
assert.throws(Test262Error, () => {
  assert.compareArray([,], [,,]);
}, 'Sparse arrays of differing lengths are not equivalent.');
assert.throws(Test262Error, () => {
  assert.compareArray([,,], [,]);
}, 'Sparse arrays of differing lengths are not equivalent.');
assert.throws(Test262Error, () => {
  assert.compareArray([,], []);
}, 'Sparse arrays are not equivalent to empty arrays.');
assert.throws(Test262Error, () => {
  assert.compareArray([], [,]);
}, 'Sparse arrays are not equivalent to empty arrays.');
