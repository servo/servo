// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/

/* Capture group reset to undefined during second iteration, so backreference doesn't see prior result. */
var re = /(?:^(a)|\1(a)|(ab)){2}/;
var str = 'aab';
var actual = re.exec(str);

assert.compareArray(actual, ['aa', undefined, 'a', undefined]);
assert.sameValue(actual.index, 0);
assert.sameValue(actual.input, str);
