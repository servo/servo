// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-buffer-byteoffset-length
description: >
  Throws a RangeError for resizable ArrayBuffers when offset > byteLength
includes: [testTypedArray.js]
features: [TypedArray, resizable-arraybuffer]
---*/

testWithTypedArrayConstructors(function(TA) {
  var BPE = TA.BYTES_PER_ELEMENT;
  var buffer = new ArrayBuffer(BPE, {maxByteLength: BPE});

  assert.throws(RangeError, function() {
    new TA(buffer, BPE * 2);
  });

  assert.throws(RangeError, function() {
    new TA(buffer, BPE * 2, undefined);
  });
}, null, ["passthrough"]);
