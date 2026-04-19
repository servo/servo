// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype-@@tostringtag
description: >
  _TypedArray_.prototype[@@toStringTag] is inherited from %TypedArray%
  _TypedArray_.prototype has no own property @@toStringTag
includes: [testTypedArray.js]
features: [BigInt, Symbol.toStringTag, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  assert.sameValue(TA.prototype.hasOwnProperty(Symbol.toStringTag), false);
}, null, ["passthrough"]);
