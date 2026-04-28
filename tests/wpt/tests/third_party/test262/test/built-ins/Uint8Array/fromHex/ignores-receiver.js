// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.fromhex
description: Uint8Array.fromHex ignores its receiver
features: [uint8array-base64, TypedArray]
---*/

var fromHex = Uint8Array.fromHex;
var noReceiver = fromHex("aa");
assert.sameValue(Object.getPrototypeOf(noReceiver), Uint8Array.prototype);

class Subclass extends Uint8Array {
  constructor() {
    throw new Test262Error("subclass constructor called");
  }
}
var fromSubclass = Subclass.fromHex("aa");
assert.sameValue(Object.getPrototypeOf(fromSubclass), Uint8Array.prototype);
