// Copyright (C) 2023 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Jordan Harband
esid: sec-arraysetlength
description: >
  Setting an invalid array length throws a RangeError
info: |
  ArraySetLength ( A, Desc )

  [...]
  5. If SameValueZero(newLen, numberLen) is false, throw a RangeError exception.
  [...]
---*/

assert.throws(RangeError, function () {
  Object.defineProperty([], 'length', { value: -1, configurable: true });
});

assert.throws(RangeError, function () {
  // the string is intentionally "computed" here to ensure there are no optimization bugs
  Object.defineProperty([], 'len' + 'gth', { value: -1, configurable: true });
});
