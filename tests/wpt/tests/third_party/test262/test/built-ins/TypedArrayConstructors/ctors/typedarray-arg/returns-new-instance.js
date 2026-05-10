// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-typedarray
description: >
  Return a TypedArray object
info: |
  22.2.4.3 TypedArray ( typedArray )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has a [[TypedArrayName]] internal slot.

  ...
  20. Return O.

includes: [testTypedArray.js]
features: [TypedArray]
---*/

var len = 10;
var typedArraySample = new Int8Array(len);

testWithTypedArrayConstructors(function(TA) {
  var typedArray = new TA(typedArraySample);

  assert.notSameValue(typedArray, typedArraySample);
  assert.sameValue(typedArray.length, len);
  assert.sameValue(typedArray.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(typedArray), TA.prototype);
}, null, ["passthrough"]);
