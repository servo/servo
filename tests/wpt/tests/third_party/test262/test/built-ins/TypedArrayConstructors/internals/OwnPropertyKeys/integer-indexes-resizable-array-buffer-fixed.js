// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-integer-indexed-exotic-objects-ownpropertykeys
description: returned keys reflect resized ArrayBuffer for a fixed-sized TypedArray
info: |
  9.4.5.6 [[OwnPropertyKeys]] ()

  ...
  3. Let getBufferByteLength be !
     MakeIdempotentArrayBufferByteLengthGetter(SeqCst).
  4. Let len be IntegerIndexedObjectLength(O, getBufferByteLength).
  5. For each integer i starting with 0 such that i < len, in ascending order,
    a. Add ! ToString(i) as the last element of keys.
  ...
includes: [testTypedArray.js]
features: [Reflect, TypedArray, resizable-arraybuffer]
---*/

// If the host chooses to throw as allowed by the specification, the observed
// behavior will be identical to the case where `ArrayBuffer.prototype.resize`
// has not been implemented. The following assertion prevents this test from
// passing in runtimes which have not implemented the method.
assert.sameValue(typeof ArrayBuffer.prototype.resize, "function");

testWithTypedArrayConstructors(function(TA) {
  var BPE = TA.BYTES_PER_ELEMENT;
  var ab = new ArrayBuffer(BPE * 4, {maxByteLength: BPE * 5});
  var array = new TA(ab, BPE, 2);

  assert.sameValue(Reflect.ownKeys(array).join(","), "0,1", "initial");

  try {
    ab.resize(BPE * 5);
  } catch (_) {}

  assert.sameValue(Reflect.ownKeys(array).join(","), "0,1", "following grow");

  try {
    ab.resize(BPE * 3);
  } catch (_) {}

  assert.sameValue(
    Reflect.ownKeys(array).join(","), "0,1", "following shrink (within bounds)"
  );

  var expected;
  try {
    ab.resize(BPE * 2);
    expected = "";
  } catch (_) {
    expected = "0,1";
  }

  assert.sameValue(
    Reflect.ownKeys(array).join(","), expected, "following shrink (out of bounds)"
  );
}, null, ["passthrough"]);
