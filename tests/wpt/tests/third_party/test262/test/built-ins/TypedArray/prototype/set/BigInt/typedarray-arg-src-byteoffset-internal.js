// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  Uses typedArray's internal [[ByteOffset]]
info: |
  22.2.3.23.2 %TypedArray%.prototype.set(typedArray [ , offset ] )
  1. Assert: typedArray has a [[TypedArrayName]] internal slot. If it does not,
  the definition in 22.2.3.23.1 applies.
  ...
  21. Let srcByteOffset be typedArray.[[ByteOffset]].
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var getCalls = 0;
var desc = {
  get: function getLen() {
    getCalls++;
    return 0;
  }
};

Object.defineProperty(TypedArray.prototype, "byteOffset", desc);

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));
  var src = new TA(makeCtorArg([42n, 43n]));
  var other = TA === BigInt64Array ? BigUint64Array : BigInt64Array;
  var src2 = new other([42n, 43n]);
  var src3 = new other(sample.buffer, 0, 2);

  Object.defineProperty(TA.prototype, "byteOffset", desc);
  Object.defineProperty(src, "byteOffset", desc);
  Object.defineProperty(src2, "byteOffset", desc);
  Object.defineProperty(src3, "byteOffset", desc);

  sample.set(src);
  sample.set(src2);
  sample.set(src3);

  assert.sameValue(getCalls, 0, "ignores byteOffset properties");
}, null, ["passthrough"]);
