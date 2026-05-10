// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Uses default constructor is `constructor` property is undefined.
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  13. Let ctor be SpeciesConstructor(O, %ArrayBuffer%).
  14. ReturnIfAbrupt(ctor).
  ...

  7.3.20 SpeciesConstructor ( O, defaultConstructor )
    ...
    2. Let C be Get(O, "constructor").
    3. ReturnIfAbrupt(C).
    4. If C is undefined, return defaultConstructor.
    ...
---*/

var arrayBuffer = new ArrayBuffer(8);
arrayBuffer.constructor = undefined;

var result = arrayBuffer.slice();
assert.sameValue(Object.getPrototypeOf(result), ArrayBuffer.prototype);
