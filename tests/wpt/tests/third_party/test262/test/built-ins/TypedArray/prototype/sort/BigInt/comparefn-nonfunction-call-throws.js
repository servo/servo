// Copyright (C) 2017 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.sort
description: throws on a non-undefined non-function
info: |
  22.2.3.26 %TypedArray%.prototype.sort ( comparefn )

  Upon entry, the following steps are performed to initialize evaluation
  of the sort function. These steps are used instead of the entry steps
  in 22.1.3.25:

  ...
  1. If _comparefn_ is not *undefined* and IsCallable(_comparefn_) is *false*, throw a *TypeError* exception.
  ...

includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42n, 43n, 44n, 45n, 46n]));

  assert.throws(TypeError, function() {
    sample.sort(null);
  });

  assert.throws(TypeError, function() {
    sample.sort(true);
  });

  assert.throws(TypeError, function() {
    sample.sort(false);
  });

  assert.throws(TypeError, function() {
    sample.sort('');
  });

  assert.throws(TypeError, function() {
    sample.sort(/a/g);
  });

  assert.throws(TypeError, function() {
    sample.sort(42);
  });

  assert.throws(TypeError, function() {
    sample.sort([]);
  });

  assert.throws(TypeError, function() {
    sample.sort({});
  });
});
