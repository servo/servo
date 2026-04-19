// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype-@@tostringtag
description: Return undefined when `this` is not Object
info: |
  22.2.3.32 get %TypedArray%.prototype [ @@toStringTag ]

  1. Let O be the this value.
  2. If Type(O) is not Object, return undefined.
  ...
includes: [testTypedArray.js]
features: [Symbol, Symbol.toStringTag, TypedArray]
---*/

var TypedArrayPrototype = TypedArray.prototype;
var getter = Object.getOwnPropertyDescriptor(
  TypedArrayPrototype, Symbol.toStringTag
).get;

assert.sameValue(getter.call(undefined), undefined, "this is undefined");
assert.sameValue(getter.call(42), undefined, "this is 42");
assert.sameValue(getter.call("foo"), undefined, "this is a string");
assert.sameValue(getter.call(true), undefined, "this is true");
assert.sameValue(getter.call(false), undefined, "this is false");
assert.sameValue(getter.call(Symbol("s")), undefined, "this is a Symbol");
assert.sameValue(getter.call(null), undefined, "this is null");
