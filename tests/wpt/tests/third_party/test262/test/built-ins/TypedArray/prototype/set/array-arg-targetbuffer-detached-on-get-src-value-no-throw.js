// Copyright (C) 2022 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Does not throw if target TA is detached mid-iteration
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([1, 2, 3]));
  var obj = {
    length: 3,
    "0": 42
  };
  Object.defineProperty(obj, 1, {
    get: function() {
      $DETACHBUFFER(sample.buffer);
    }
  });
  let get2Called = false;
  Object.defineProperty(obj, 2, {
    get: function() {
      get2Called = true;
      return 2;
    }
  });

  sample.set(obj);

  assert.sameValue(true, get2Called);
  assert.sameValue(0, sample.byteLength);
  assert.sameValue(0, sample.byteOffset);
  assert.sameValue(0, sample.length);
}, null, null, ["immutable"]);
