// Copyright (C) 2015 André Bargull. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Throws TypeError if `constructor` property is not an object.
info: |
  SharedArrayBuffer.prototype.slice ( start, end )

features: [SharedArrayBuffer, Symbol]
---*/

var arrayBuffer = new SharedArrayBuffer(8);

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
