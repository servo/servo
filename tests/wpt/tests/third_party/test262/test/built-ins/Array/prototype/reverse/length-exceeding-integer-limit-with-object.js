// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reverse
description: >
  Ensure reverse() implementation correctly handles length exceeding 2^53-1 with plain objects.
info: |
  ...
  2. Let len be ? ToLength(? Get(O, "length")).
  ...
features: [exponentiation]
---*/

function StopReverse() {}

// Object with large "length" property and no indexed properties in the uint32 range.
var arrayLike = {
  get "9007199254740990" () {
    throw new StopReverse();
  },
  get "9007199254740991" () {
    throw new Test262Error("Get 9007199254740991");
  },
  get "9007199254740992" () {
    throw new Test262Error("Get 9007199254740992");
  },
  length: 2 ** 53 + 2,
};

assert.throws(StopReverse, function() {
  Array.prototype.reverse.call(arrayLike);
});
