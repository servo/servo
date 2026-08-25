// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.join
description: Throws a TypeError if this has a detached buffer
info: |
  %TypedArray%.prototype.join ( separator )

  The interpretation and use of the arguments of %TypedArray%.prototype.join are the same as for Array.prototype.join as defined in 22.1.3.15.

  When the join method is called with one argument separator, the following steps are taken:

  Let O be the this value.
  Perform ? ValidateTypedArray(O).
  ...

includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

let obj = {
  toString() {
    throw new Test262Error();
  }
};

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  let sample = new TA(makeCtorArg(1));
  $DETACHBUFFER(sample.buffer);
  assert.throws(TypeError, () => {
    sample.join(obj);
  });
}, null, null, ["immutable"]);
