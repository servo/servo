// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-array-offset
description: >
  Uses target's internal [[ArrayLength]]
info: |
  22.2.3.23.1 %TypedArray%.prototype.set (array [ , offset ] )

  1. Assert: array is any ECMAScript language value other than an Object with a
  [[TypedArrayName]] internal slot. If it is such an Object, the definition in
  22.2.3.23.2 applies.
  ...
  10. Let targetLength be the value of target's [[ArrayLength]] internal slot.
  ...
  17. If srcLength + targetOffset > targetLength, throw a RangeError exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var getCalls = 0;
var desc = {
  get: function() {
    getCalls++;
    return 0;
  }
};

Object.defineProperty(TypedArray.prototype, "length", desc);

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(2));

  Object.defineProperty(TA.prototype, "length", desc);
  Object.defineProperty(sample, "length", desc);

  sample.set([42, 43]);

  assert.sameValue(getCalls, 0, "ignores length properties");
}, null, ["passthrough"]);
