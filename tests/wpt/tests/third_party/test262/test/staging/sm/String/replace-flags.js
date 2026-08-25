// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var BUGNUMBER = 1108382;
var summary = 'Remove non-standard flag argument from String.prototype.{search,match,replace}.';

var result = "bbbAa".match("a", "i");
assert.sameValue(result.index, 4);
assert.sameValue(result.length, 1);
assert.sameValue(result[0], "a");

result = "bbbA".match("a", "i");
assert.sameValue(result, null);

result = "bbbAa".search("a", "i");
assert.sameValue(result, 4);

result = "bbbA".search("a", "i");
assert.sameValue(result, -1);

result = "bbbAaa".replace("a", "b", "g");
assert.sameValue(result, "bbbAba");

