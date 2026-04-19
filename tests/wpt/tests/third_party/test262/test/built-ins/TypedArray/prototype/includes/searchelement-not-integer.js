// Copyright (C) 2020 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.includes
description: Check that search element is not coerced if not an integer
info: |
  22.2.3.13 %TypedArray%.prototype.includes ( searchElement [ , fromIndex ] )
  
  %TypedArray%.prototype.includes is a distinct function that implements the same algorithm as Array.prototype.includes as defined in 22.1.3.13 

  22.1.3.13 Array.prototype.includes ( searchElement [ , fromIndex ] )

  8. Repeat, while k < len
    a. Let elementK be the result of ? Get(O, ! ToString(k)).
    b. If SameValueZero(searchElement, elementK) is true, return true.

includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(10));
  function throwFunc(){
    throw Test262Error()
    return 0;
  }

    assert.sameValue(sample.includes({valueOf : throwFunc}), false);
});
