// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.indexof
description: Return abrupt from ToInteger(fromIndex)
info: |
  22.2.3.13 %TypedArray%.prototype.indexOf (searchElement [ , fromIndex ] )

  %TypedArray%.prototype.indexOf is a distinct function that implements the same
  algorithm as Array.prototype.indexOf as defined in 22.1.3.12 except that the
  this object's [[ArrayLength]] internal slot is accessed in place of performing
  a [[Get]] of "length".

  22.1.3.12 Array.prototype.indexOf ( searchElement [ , fromIndex ] )

  ...
  4. Let n be ? ToInteger(fromIndex). (If fromIndex is undefined, this step
  produces the value 0.)
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var fromIndex = {
  valueOf: function() {
    throw new Test262Error();
  }
};

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));

  assert.throws(Test262Error, function() {
    sample.indexOf(7, fromIndex);
  });
});
