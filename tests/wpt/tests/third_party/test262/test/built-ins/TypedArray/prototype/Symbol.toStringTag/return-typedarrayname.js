// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype-@@tostringtag
description: >
  Return value from the [[TypedArrayName]] internal slot
info: |
  22.2.3.32 get %TypedArray%.prototype [ @@toStringTag ]

  ...
  4. Let name be the value of O's [[TypedArrayName]] internal slot.
  5. Assert: name is a String value.
  6. Return name.
includes: [testTypedArray.js]
features: [Symbol.toStringTag, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var ta = new TA();
  assert.sameValue(ta[Symbol.toStringTag], TA.name, "property value");
}, null, ["passthrough"]);
