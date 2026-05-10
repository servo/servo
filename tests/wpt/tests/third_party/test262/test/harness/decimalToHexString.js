// Copyright (C) 2017 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Including decimalToHexString.js will expose two functions:

      decimalToHexString
      decimalToPercentHexString

includes: [decimalToHexString.js]
---*/

assert.sameValue(decimalToHexString(-1), "FFFFFFFF");
assert.sameValue(decimalToHexString(0.5), "0000");
assert.sameValue(decimalToHexString(1), "0001");
assert.sameValue(decimalToHexString(100), "0064");
assert.sameValue(decimalToHexString(65535), "FFFF");
assert.sameValue(decimalToHexString(65536), "10000");

assert.sameValue(decimalToPercentHexString(-1), "%FF");
assert.sameValue(decimalToPercentHexString(0.5), "%00");
assert.sameValue(decimalToPercentHexString(1), "%01");
assert.sameValue(decimalToPercentHexString(100), "%64");
assert.sameValue(decimalToPercentHexString(65535), "%FF");
assert.sameValue(decimalToPercentHexString(65536), "%00");
