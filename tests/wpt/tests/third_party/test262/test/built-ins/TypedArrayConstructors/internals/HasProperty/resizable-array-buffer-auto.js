// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-hasproperty-p
description: returned keys reflect resized ArrayBuffer for a dynamically-sized TypedArray
includes: [testTypedArray.js]
features: [Reflect, TypedArray, resizable-arraybuffer]
---*/

// If the host chooses to throw as allowed by the specification, the observed
// behavior will be identical to the case where `ArrayBuffer.prototype.resize`
// has not been implemented. The following assertion prevents this test from
// passing in runtimes which have not implemented the method.
assert.sameValue(typeof ArrayBuffer.prototype.resize, "function");

function inspect(array) {
  return [
    Reflect.has(array, 0),
    Reflect.has(array, 1),
    Reflect.has(array, 2),
    Reflect.has(array, 3),
    Reflect.has(array, 4)
  ].join(",");
}

testWithTypedArrayConstructors(function(TA) {
  var BPE = TA.BYTES_PER_ELEMENT;
  var ab = new ArrayBuffer(BPE * 4, {maxByteLength: BPE * 5});
  var array = new TA(ab, BPE);
  var expected = "true,true,true,false,false";

  assert.sameValue(inspect(array), expected, "initial");

  try {
    ab.resize(BPE * 5);
    expected = "true,true,true,true,false";
  } catch (_) {}

  assert.sameValue(inspect(array), expected, "following grow");

  try {
    ab.resize(BPE * 3);
    expected = "true,true,false,false,false";
  } catch (_) {}

  assert.sameValue(inspect(array), expected, "following shrink (within bounds)");

  try {
    ab.resize(BPE);
    expected = "false,false,false,false,false";
  } catch (_) {}

  assert.sameValue(inspect(array), expected, "following shrink (on boundary)");

  try {
    ab.resize(0);
    expected = "false,false,false,false,false";
  } catch (_) {}

  assert.sameValue(inspect(array), expected, "following shrink (out of bounds)");
}, null, ["passthrough"]);
