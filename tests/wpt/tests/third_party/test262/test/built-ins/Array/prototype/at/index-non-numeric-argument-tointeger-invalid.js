// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.at
description: >
  Property type and descriptor.
info: |
  Array.prototype.at( index )

  Let relativeIndex be ? ToInteger(index).

features: [Array.prototype.at]
---*/
assert.sameValue(
  typeof Array.prototype.at,
  'function',
  'The value of `typeof Array.prototype.at` is expected to be "function"'
);

let a = [0,1,2,3];

assert.throws(TypeError, () => {
  a.at(Symbol());
}, 'a.at(Symbol()) throws a TypeError exception');
