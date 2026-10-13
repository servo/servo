// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.fill
description: >
  Fills all the elements from a with a custom end index.
info: |
  22.2.3.8 %TypedArray%.prototype.fill (value [ , start [ , end ] ] )

  %TypedArray%.prototype.fill is a distinct function that implements the same
  algorithm as Array.prototype.fill as defined in 22.1.3.6 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length". The implementation of the algorithm may be optimized with
  the knowledge that the this value is an object that has a fixed length and
  whose integer indexed properties are not sparse. However, such optimization
  must not introduce any observable changes in the specified behaviour of the
  algorithm.

  ...

  22.1.3.6 Array.prototype.fill (value [ , start [ , end ] ] )

  ...
  5. If end is undefined, let relativeEnd be len; else let relativeEnd be ?
  ToInteger(end).
  6. If relativeEnd < 0, let final be max((len + relativeEnd), 0); else let
  final be min(relativeEnd, len).
  ...
includes: [compareArray.js, testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  assert(
    compareArray(new TA(makeCtorArg([0n, 0n, 0n])).fill(8n, 0, 1), [8n, 0n, 0n]),
    "Fill elements from custom end position"
  );

  assert(
    compareArray(new TA(makeCtorArg([0n, 0n, 0n])).fill(8n, 0, -1), [8n, 8n, 0n]),
    "negative end sets final position to max((length + relativeEnd), 0)"
  );

  assert(
    compareArray(new TA(makeCtorArg([0n, 0n, 0n])).fill(8n, 0, 5), [8n, 8n, 8n]),
    "end position is never higher than of length"
  );

  assert(
    compareArray(new TA(makeCtorArg([0n, 0n, 0n])).fill(8n, 0, -4), [0n, 0n, 0n]),
    "end position is 0 when (len + relativeEnd) < 0"
  );
}, null, null, ["immutable"]);
