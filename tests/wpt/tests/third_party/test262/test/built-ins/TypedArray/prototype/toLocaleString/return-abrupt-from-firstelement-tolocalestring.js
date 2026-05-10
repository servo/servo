// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.tolocalestring
description: Returns abrupt from firstElement's toLocaleString
info: |
  22.2.3.28 %TypedArray%.prototype.toLocaleString ([ reserved1 [ , reserved2 ] ])

  %TypedArray%.prototype.toLocaleString is a distinct function that implements
  the same algorithm as Array.prototype.toLocaleString as defined in 22.1.3.27
  except that the this object's [[ArrayLength]] internal slot is accessed in
  place of performing a [[Get]] of "length".
  
  22.1.3.27 Array.prototype.toLocaleString ( [ reserved1 [ , reserved2 ] ] )
  
  ...
  4. If len is zero, return the empty String.
  5. Let firstElement be ? Get(array, "0").
  6. If firstElement is undefined or null, then
    a. Let R be the empty String.
  7. Else,
    a. Let R be ? ToString(? Invoke(firstElement, "toLocaleString")).
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var calls;

Number.prototype.toLocaleString = function() {
  calls++;
  throw new Test262Error();
};

var arr = [42, 0];

testWithTypedArrayConstructors(function(TA) {
  calls = 0;
  var sample = new TA(arr);
  assert.throws(Test262Error, function() {
    sample.toLocaleString();
  });
  assert.sameValue(calls, 1, "abrupt from first element");
}, null, ["passthrough"]);
