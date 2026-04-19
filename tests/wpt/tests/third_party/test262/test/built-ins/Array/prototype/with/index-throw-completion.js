// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.with
description: >
  Index coercion returns a throw completion.
info: |
  Array.prototype.with ( index, value )

  ...
  4. Let relativeIndex be ? ToIntegerOrInfinity(index).
  ...
features: [change-array-by-copy]
---*/

function MyError() {}

var index = {
  valueOf() {
    throw new MyError();
  }
};

assert.throws(MyError, function() {
  [].with(index, null);
});
