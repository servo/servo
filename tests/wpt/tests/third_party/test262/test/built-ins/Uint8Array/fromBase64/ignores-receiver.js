// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.frombase64
description: Uint8Array.fromBase64 ignores its receiver
features: [uint8array-base64, TypedArray]
---*/

var fromBase64 = Uint8Array.fromBase64;
var noReceiver = fromBase64("Zg==");
assert.sameValue(Object.getPrototypeOf(noReceiver), Uint8Array.prototype);

class Subclass extends Uint8Array {
  constructor() {
    throw new Test262Error("subclass constructor called");
  }
}
var fromSubclass = Subclass.fromBase64("Zg==");
assert.sameValue(Object.getPrototypeOf(fromSubclass), Uint8Array.prototype);
