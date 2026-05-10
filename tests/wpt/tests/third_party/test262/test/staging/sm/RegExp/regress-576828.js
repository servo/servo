// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/

var re = /(z\1){3}/;
var str = 'zzz';
var actual = re.exec(str);

assert.compareArray(actual, ['zzz', 'z']);
assert.sameValue(actual.index, 0);
assert.sameValue(actual.input, str);
