// Copyright (C) 2018 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.sort
description: The result of compareFn is immediately passed through ToNumber
info: |
  22.2.3.26 %TypedArray%.prototype.sort ( comparefn )

  ...
  2. If comparefn is not undefined, then
    a. Let v be ? ToNumber(? Call(comparefn, undefined, « x, y »)).
    b. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
    ...
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var ta = new TA(4);
  var ab = ta.buffer;

  var called = false;
  ta.sort(function(a, b) {
    // Detaching the buffer does not cause sort to throw.
    $DETACHBUFFER(ab);
    return {
      [Symbol.toPrimitive]() { called = true; }
    };
  });

  assert.sameValue(true, called);
}, null, ["passthrough"]);
