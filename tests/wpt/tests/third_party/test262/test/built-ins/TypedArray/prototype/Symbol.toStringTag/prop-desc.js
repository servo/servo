// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype-@@tostringtag
description: >
  "@@toStringTag" property of TypedArrayPrototype
info: |
  22.2.3.31 get %TypedArray%.prototype [ @@toStringTag ]

  %TypedArray%.prototype[@@toStringTag] is an accessor property whose set
  accessor function is undefined.
  ...

  This property has the attributes { [[Enumerable]]: false, [[Configurable]]:
  true }.
includes: [propertyHelper.js, testTypedArray.js]
features: [Symbol.toStringTag]
---*/

var TypedArrayPrototype = TypedArray.prototype;
var desc = Object.getOwnPropertyDescriptor(
  TypedArrayPrototype, Symbol.toStringTag
);

assert.sameValue(desc.set, undefined);
assert.sameValue(typeof desc.get, 'function');
verifyNotEnumerable(TypedArrayPrototype, Symbol.toStringTag);
verifyConfigurable(TypedArrayPrototype, Symbol.toStringTag);
