// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-buffer-byteoffset-length
description: >
  Use prototype from %TypedArray% if newTarget's prototype is not an Object
info: |
  22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has an [[ArrayBufferData]] internal slot.

  ...
  4. Let O be ? AllocateTypedArray(constructorName, NewTarget,
  %TypedArrayPrototype%).
  ...

  22.2.4.2.1 Runtime Semantics: AllocateTypedArray (constructorName, newTarget,
  defaultProto [ , length ])

  1. Let proto be ? GetPrototypeFromConstructor(newTarget, defaultProto).
  2. Let obj be IntegerIndexedObjectCreate (proto, «[[ViewedArrayBuffer]],
  [[TypedArrayName]], [[ByteLength]], [[ByteOffset]], [[ArrayLength]]» ).
  ...

  9.4.5.7 IntegerIndexedObjectCreate (prototype, internalSlotsList)

  ...
  10. Set the [[Prototype]] internal slot of A to prototype.
  ...
  12. Return A.
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var buffer = new ArrayBuffer(8);

function newTarget() {}
newTarget.prototype = null;

testWithBigIntTypedArrayConstructors(function(TA) {
  var ta = Reflect.construct(TA, [buffer], newTarget);

  assert.sameValue(ta.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(ta), TA.prototype);
}, null, ["passthrough"]);
