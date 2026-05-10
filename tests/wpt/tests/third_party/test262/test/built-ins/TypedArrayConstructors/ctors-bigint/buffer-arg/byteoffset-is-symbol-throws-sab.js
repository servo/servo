// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-buffer-byteoffset-length
description: >
  Return abrupt from parsing integer value from byteOffset as a symbol
info: |
  22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has an [[ArrayBufferData]] internal slot.

  ...
  7. Let offset be ? ToInteger(byteOffset).
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, SharedArrayBuffer, TypedArray]
---*/

var byteOffset = Symbol("1");
var buffer = new SharedArrayBuffer(8);

testWithBigIntTypedArrayConstructors(function(TA) {
  assert.throws(TypeError, function() {
    new TA(buffer, byteOffset);
  });
}, null, ["passthrough"]);
