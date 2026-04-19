// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/

/* Back reference is actually a forwards reference. */
var re = /(\2(a)){2}/;
var str = 'aaa';
var actual = re.exec(str);

assert.compareArray(actual, ['aa', 'a', 'a']);
assert.sameValue(actual.index, 0);
assert.sameValue(actual.input, str);
