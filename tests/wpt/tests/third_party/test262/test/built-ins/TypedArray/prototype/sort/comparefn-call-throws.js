// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.sort
description: Returns abrupt from comparefn
info: |
  22.2.3.26 %TypedArray%.prototype.sort ( comparefn )

  When the TypedArray SortCompare abstract operation is called with two
  arguments x and y, the following steps are taken:

  ...
  2. If the argument comparefn is not undefined, then
    a. Let v be ? Call(comparefn, undefined, « x, y »).
    ...
  ...

  22.1.3.25 Array.prototype.sort (comparefn)

  The following steps are taken:

  - If an abrupt completion is returned from any of these operations, it is
  immediately returned as the value of this function.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([42, 43, 44, 45, 46]));
  var calls = 0;

  var comparefn = function() {
    calls += 1;
    throw new Test262Error();
  };

  assert.throws(Test262Error, function() {
    sample.sort(comparefn);
  });

  assert.sameValue(calls, 1, "immediately returned");
});
