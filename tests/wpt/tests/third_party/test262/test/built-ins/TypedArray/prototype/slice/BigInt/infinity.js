// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: Infinity values on start and end
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40n, 41n, 42n, 43n]));

  assert(
    compareArray(sample.slice(-Infinity), [40n, 41n, 42n, 43n]),
    "start == -Infinity"
  );
  assert(
    compareArray(sample.slice(Infinity), []),
    "start == Infinity"
  );
  assert(
    compareArray(sample.slice(0, -Infinity), []),
    "end == -Infinity"
  );
  assert(
    compareArray(sample.slice(0, Infinity), [40n, 41n, 42n, 43n]),
    "end == Infinity"
  );
});
