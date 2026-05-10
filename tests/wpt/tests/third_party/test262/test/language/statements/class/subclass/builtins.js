// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class sublclassing builtins
---*/
class ExtendedUint8Array extends Uint8Array {
  constructor() {
    super(10);
    this[0] = 255;
    this[1] = 0xFFA;
  }
}

var eua = new ExtendedUint8Array();
assert.sameValue(eua.length, 10, "The value of `eua.length` is `10`");
assert.sameValue(eua.byteLength, 10, "The value of `eua.byteLength` is `10`");
assert.sameValue(eua[0], 0xFF, "The value of `eua[0]` is `0xFF`");
assert.sameValue(eua[1], 0xFA, "The value of `eua[1]` is `0xFA`");
assert.sameValue(
  Object.getPrototypeOf(eua),
  ExtendedUint8Array.prototype,
  "`Object.getPrototypeOf(eua)` returns `ExtendedUint8Array.prototype`"
);
assert.sameValue(
  Object.prototype.toString.call(eua),
  "[object Uint8Array]",
  "`Object.prototype.toString.call(eua)` returns `\"[object Uint8Array]\"`"
);
