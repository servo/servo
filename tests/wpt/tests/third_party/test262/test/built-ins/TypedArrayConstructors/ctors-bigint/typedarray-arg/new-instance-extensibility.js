// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-typedarray
description: >
  The new typedArray instance from a typedArray argument is extensible
info: |
  22.2.4.3 TypedArray ( typedArray )

  ...
  4. Let O be ? AllocateTypedArray(constructorName, NewTarget,
  "%TypedArrayPrototype%").
  ...

  22.2.4.2.1 Runtime Semantics: AllocateTypedArray (constructorName, newTarget,
  defaultProto [ , length ])

  ...
  2. Let obj be IntegerIndexedObjectCreate(proto, « [[ViewedArrayBuffer]],
  [[TypedArrayName]], [[ByteLength]], [[ByteOffset]], [[ArrayLength]] »).
  ...

  9.4.5.7 IntegerIndexedObjectCreate (prototype, internalSlotsList)

  ...
  11. Set the [[Extensible]] internal slot of A to true.
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var typedArraySample1 = new BigInt64Array();
var typedArraySample2 = new BigInt64Array();
Object.preventExtensions(typedArraySample2);

testWithBigIntTypedArrayConstructors(function(TA) {
  var sample1 = new TA(typedArraySample1);

  assert(Object.isExtensible(sample1), "new instance is extensible");

  var sample2 = new TA(typedArraySample2);
  assert(
    Object.isExtensible(sample2),
    "new instance does not inherit extensibility from typedarray argument"
  );
}, null, ["passthrough"]);
