// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-buffer-byteoffset-length
description: >
  Throws a RangeError if ToInteger(byteOffset) is < 0
info: |
  22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has an [[ArrayBufferData]] internal slot.

  ...
  7. Let offset be ? ToInteger(byteOffset).
  8. If offset < 0, throw a RangeError exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var buffer = new ArrayBuffer(8);

testWithTypedArrayConstructors(function(TA) {
  assert.throws(RangeError, function() {
    new TA(buffer, -1);
  });
  assert.throws(RangeError, function() {
    new TA(buffer, -Infinity);
  });
}, null, ["passthrough"]);
