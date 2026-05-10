// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.tolocalestring
description:  Get "length" uses internal ArrayLength
info: |
  22.2.3.28 %TypedArray%.prototype.toLocaleString ([ reserved1 [ , reserved2 ] ])

  %TypedArray%.prototype.toLocaleString is a distinct function that implements
  the same algorithm as Array.prototype.toLocaleString as defined in 22.1.3.27
  except that the this object's [[ArrayLength]] internal slot is accessed in
  place of performing a [[Get]] of "length".

  22.1.3.27 Array.prototype.toLocaleString ( [ reserved1 [ , reserved2 ] ] )

  1. Let array be ? ToObject(this value).
  2.Let len be ? ToLength(? Get(array, "length")).
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

Object.defineProperty(TypedArray.prototype, "length", desc);

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n]));

  Object.defineProperty(TA.prototype, "length", desc);
  Object.defineProperty(sample, "length", desc);

  sample.toLocaleString();

  assert.sameValue(getCalls, 0, "ignores length properties");
}, null, ["passthrough"]);
