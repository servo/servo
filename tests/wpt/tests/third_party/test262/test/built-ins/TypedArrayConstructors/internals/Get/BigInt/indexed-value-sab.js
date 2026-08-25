// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-get-p-receiver
description: >
  Return value from valid numeric index, with SharedArrayBuffer
includes: [testTypedArray.js]
features: [BigInt, TypedArray, SharedArrayBuffer]
---*/

var proto = TypedArray.prototype;
var throwDesc = {
  get: function() {
    throw new Test262Error("OrdinaryGet was called! Ref: 9.1.8.1 3.c");
  }
};
Object.defineProperty(proto, "0", throwDesc);
Object.defineProperty(proto, "1", throwDesc);

testWithBigIntTypedArrayConstructors(function(TA) {
  var sab = new SharedArrayBuffer(TA.BYTES_PER_ELEMENT * 2);
  var sample = new TA(sab);
  sample.set([42n, 1n]);

  assert.sameValue(sample["0"], 42n);
  assert.sameValue(sample["1"], 1n);
}, null, ["passthrough"]);
