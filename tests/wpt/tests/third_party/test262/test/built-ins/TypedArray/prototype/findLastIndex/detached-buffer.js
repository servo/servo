// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlastindex
description: Throws a TypeError if this has a detached buffer
info: |
  %TypedArray%.prototype.findLastIndex ( predicate [ , thisArg ] )

  ...
  2. Perform ? ValidateTypedArray(O).
  ...

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  ...
  5. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray, array-find-from-last]
---*/

var predicate = function() {
  throw new Test262Error();
};

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));
  $DETACHBUFFER(sample.buffer);
  assert.throws(TypeError, function() {
    sample.findLastIndex(predicate);
  });
}, null, null, ["immutable"]);
