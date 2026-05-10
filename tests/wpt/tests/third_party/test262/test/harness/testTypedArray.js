// Copyright (c) 2017 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Including testTypedArray.js will expose:

        var typedArrayConstructors = [ array of TypedArray constructors ]
        var TypedArray

        testWithTypedArrayConstructors()
        testTypedArrayConversions()

includes: [compareArray.js, testTypedArray.js]
features: [TypedArray]
---*/

assert(typeof TypedArray === "function");
assert.sameValue(TypedArray, Object.getPrototypeOf(Uint8Array));

var hasFloat16Array = typeof Float16Array !== "undefined";
var expectCtors = [
  Float64Array,
  Float32Array,
  hasFloat16Array ? Float16Array : undefined,
  Int32Array,
  Int16Array,
  Int8Array,
  Uint32Array,
  Uint16Array,
  Uint8Array,
  Uint8ClampedArray
];
if (!hasFloat16Array) expectCtors.splice(2, 1);
assert.compareArray(typedArrayConstructors, expectCtors, "typedArrayConstructors");

var callCounts = {};
var totalCallCount = 0;
testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var name = TA.name;
  callCounts[name] = (callCounts[name] || 0) + 1;
  totalCallCount++;
});
assert(totalCallCount > typedArrayConstructors.length, "total call count");

var expectEachCallCount = totalCallCount / typedArrayConstructors.length;
for (var i = 0; i < typedArrayConstructors.length; i++) {
  var name = typedArrayConstructors[i].name;
  assert.sameValue(callCounts[name], expectEachCallCount, name + " call count");
}
