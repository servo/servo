// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  Uses typedArray's internal [[ArrayLength]]
info: |
  22.2.3.23.2 %TypedArray%.prototype.set(typedArray [ , offset ] )

  1. Assert: typedArray has a [[TypedArrayName]] internal slot. If it does not,
  the definition in 22.2.3.23.1 applies.
  ...
  20. Let srcLength be the value of typedArray's [[ArrayLength]] internal slot.
  ...
  22. If srcLength + targetOffset > targetLength, throw a RangeError exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var getCalls = 0;
var desc = {
  get: function getLen() {
    getCalls++;
    return 42;
  }
};

Object.defineProperty(TypedArray.prototype, "length", desc);

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));
  var src = new TA(makeCtorArg([42, 43]));

  Object.defineProperty(TA.prototype, "length", desc);
  Object.defineProperty(src, "length", desc);

  sample.set(src);

  assert.sameValue(getCalls, 0, "ignores length properties");
}, null, ["passthrough"]);
