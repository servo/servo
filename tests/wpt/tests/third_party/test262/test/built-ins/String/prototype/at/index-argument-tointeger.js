// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.at
description: >
  Property type and descriptor.
info: |
  String.prototype.at( index )

  Let relativeIndex be ? ToInteger(index).

features: [String.prototype.at]
---*/
assert.sameValue(typeof String.prototype.at, 'function');

let valueOfCallCount = 0;
let index = {
  valueOf() {
    valueOfCallCount++;
    return 1;
  }
};

let s = "01";

assert.sameValue(s.at(index), '1', 's.at({valueOf() {valueOfCallCount++; return 1;}}) must return 1');
assert.sameValue(valueOfCallCount, 1, 'The value of `valueOfCallCount` is 1');
