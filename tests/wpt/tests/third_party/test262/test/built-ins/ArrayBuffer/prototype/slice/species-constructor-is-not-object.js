// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  Throws TypeError if `constructor` property is not an object.
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
    5. If Type(C) is not Object, throw a TypeError exception.
    ...
features: [Symbol]
---*/

var arrayBuffer = new ArrayBuffer(8);

function callSlice() {
  arrayBuffer.slice();
}

arrayBuffer.constructor = null;
assert.throws(TypeError, callSlice, "`constructor` value is null");

arrayBuffer.constructor = true;
assert.throws(TypeError, callSlice, "`constructor` value is Boolean");

arrayBuffer.constructor = "";
assert.throws(TypeError, callSlice, "`constructor` value is String");

arrayBuffer.constructor = Symbol();
assert.throws(TypeError, callSlice, "`constructor` value is Symbol");

arrayBuffer.constructor = 1;
assert.throws(TypeError, callSlice, "`constructor` value is Number");
