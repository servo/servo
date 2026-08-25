// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: >
  Return abrupt from ToBigInt(symbol value)
features: [DataView, ArrayBuffer, Symbol, BigInt]
---*/

var buffer = new ArrayBuffer(8);
var sample = new DataView(buffer, 0);

var s = Symbol("1");

assert.throws(TypeError, function() {
  sample.setBigInt64(0, s);
});
