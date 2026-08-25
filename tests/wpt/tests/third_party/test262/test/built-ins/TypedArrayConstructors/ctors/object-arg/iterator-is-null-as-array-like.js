// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray-object
description: >
  If object's @@iterator is `null`, it is considered an array-like object.
info: |
  TypedArray ( object )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object does not have either a [[TypedArrayName]] or an [[ArrayBufferData]]
  internal slot.

  [...]
  5. Let usingIterator be ? GetMethod(object, @@iterator).
  6. If usingIterator is not undefined, then
    [...]
  7. NOTE: object is not an Iterable so assume it is already an array-like object.
  [...]

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
includes: [testTypedArray.js]
features: [Symbol.iterator, TypedArray]
---*/

var obj = {length: 2, 0: 1, 1: 2};
obj[Symbol.iterator] = null;

testWithTypedArrayConstructors(function(TypedArray) {
  var typedArray = new TypedArray(obj);

  assert(typedArray instanceof TypedArray);
  assert.sameValue(typedArray.length, 2);
  assert.sameValue(typedArray[0], 1);
  assert.sameValue(typedArray[1], 2);
}, null, ["passthrough"]);
