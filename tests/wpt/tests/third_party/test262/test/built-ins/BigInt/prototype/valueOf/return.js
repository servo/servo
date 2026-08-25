// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.valueof
description: >
  BigInt.prototype.valueOf returns the primitive BigInt value.
info: |
  BigInt.prototype.valueOf ( )

  Return ? thisBigIntValue(this value).
features: [BigInt]
---*/

var valueOf = BigInt.prototype.valueOf;

assert.sameValue(valueOf.call(0n), 0n, "0n");
assert.sameValue(valueOf.call(Object(0n)), 0n, "Object(0n)");
