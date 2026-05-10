// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.tolocalestring
description: Returns an empty string if called on an empty instance
info: |
  22.2.3.28 %TypedArray%.prototype.toLocaleString ([ reserved1 [ , reserved2 ] ])

  %TypedArray%.prototype.toLocaleString is a distinct function that implements
  the same algorithm as Array.prototype.toLocaleString as defined in 22.1.3.27
  except that the this object's [[ArrayLength]] internal slot is accessed in
  place of performing a [[Get]] of "length".

  22.1.3.27 Array.prototype.toLocaleString ( [ reserved1 [ , reserved2 ] ] )

  ...
  4. If len is zero, return the empty String.
  ...
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var sample = new TA();
  assert.sameValue(sample.toLocaleString(), "");
}, null, ["passthrough"]);
