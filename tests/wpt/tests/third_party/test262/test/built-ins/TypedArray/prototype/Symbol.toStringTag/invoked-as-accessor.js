// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype-@@tostringtag
description: >
  Return undefined if this value does not have a [[TypedArrayName]] internal slot
info: |
  22.2.3.31 get %TypedArray%.prototype [ @@toStringTag ]

  1. Let O be the this value.
  ...
  3. If O does not have a [[TypedArrayName]] internal slot, return undefined.
  ...
includes: [testTypedArray.js]
features: [Symbol.toStringTag, TypedArray]
---*/

var TypedArrayPrototype = TypedArray.prototype;

assert.sameValue(TypedArrayPrototype[Symbol.toStringTag], undefined);
