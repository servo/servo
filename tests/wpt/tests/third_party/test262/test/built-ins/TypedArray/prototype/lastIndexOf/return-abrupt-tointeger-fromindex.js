// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.lastindexof
description: Return abrupt from ToInteger(fromIndex)
info: |
  22.2.3.17 %TypedArray%.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  %TypedArray%.prototype.lastIndexOf is a distinct function that implements the
  same algorithm as Array.prototype.lastIndexOf as defined in 22.1.3.15 except
  that the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  22.1.3.15 Array.prototype.lastIndexOf ( searchElement [ , fromIndex ] )

  ...
  4. If argument fromIndex was passed, let n be ? ToInteger(fromIndex); else let
  n be len-1.
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
    sample.lastIndexOf(7, fromIndex);
  });
});
