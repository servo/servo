// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.transfer
description: >
  Throws a RangeError if the newLength is larger than 2^53 - 1 due to clamping
  in ToIndex.
features: [arraybuffer-transfer]
---*/

var ab = new ArrayBuffer(0);

assert.throws(RangeError, function() {
  // Math.pow(2, 53) = 9007199254740992
  ab.transfer(9007199254740992);
});
