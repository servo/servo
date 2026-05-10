// Copyright (C) 2022 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Creating TypedArray from other TypedArrays doesn't look up Symbol.species.
features: [TypedArray, ArrayBuffer, Symbol.species]
---*/

let throwOnGrossBufferConstruction = false;

class GrossBuffer extends ArrayBuffer {
  constructor() {
    super(...arguments);
    if (throwOnGrossBufferConstruction) {
      throw new Test262Error("unreachable");
    }
  }
  static get [Symbol.species]() {
    throw new Test262Error("unreachable");
  }
}

let grossBuf = new GrossBuffer(1024);
throwOnGrossBufferConstruction = true;
let grossTA = new Uint8Array(grossBuf);
let mysteryTA = new Int8Array(grossTA);

assert.sameValue(mysteryTA.buffer.__proto__, ArrayBuffer.prototype);
assert.sameValue(mysteryTA.buffer.constructor, ArrayBuffer);
