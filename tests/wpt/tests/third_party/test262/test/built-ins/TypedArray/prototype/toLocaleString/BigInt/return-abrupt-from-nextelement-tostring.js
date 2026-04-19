// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.tolocalestring
description: >
  Return abrupt from nextElement's toLocaleString => valueOf
info: |
  22.2.3.28 %TypedArray%.prototype.toLocaleString ([ reserved1 [ , reserved2 ] ])

  %TypedArray%.prototype.toLocaleString is a distinct function that implements
  the same algorithm as Array.prototype.toLocaleString as defined in 22.1.3.27
  except that the this object's [[ArrayLength]] internal slot is accessed in
  place of performing a [[Get]] of "length".

  22.1.3.27 Array.prototype.toLocaleString ( [ reserved1 [ , reserved2 ] ] )

  ...
  5. Let firstElement be ? Get(array, "0").
  6. If firstElement is undefined or null, then
    a. Let R be the empty String.
  7. Else,
    a. Let R be ? ToString(? Invoke(firstElement, "toLocaleString")).
  8. Let k be 1.
  9.Repeat, while k < len
    a. Let S be a String value produced by concatenating R and separator.
    b. Let nextElement be ? Get(array, ! ToString(k)).
    c. If nextElement is undefined or null, then
      i. Let R be the empty String.
    d. Else,
      i. Let R be ? ToString(? Invoke(nextElement, "toLocaleString")).
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

var calls = 0;

BigInt.prototype.toLocaleString = function() {
  return {
    toString: function() {
      calls++;
      if (calls > 1) {
        throw new Test262Error();
      }
    }
  };
};

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 0n]));
  calls = 0;
  assert.throws(Test262Error, function() {
    sample.toLocaleString();
  });
  assert.sameValue(calls, 2, "abrupt from a nextElement");
});
