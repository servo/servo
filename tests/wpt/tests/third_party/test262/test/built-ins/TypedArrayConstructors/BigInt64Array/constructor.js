// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-typedarray-constructors
description: BigInt64Array is a constructor function
info: |
  22.2.4 The TypedArray Constructors

  [...]

  The TypedArray intrinsic constructor functions are single functions
  whose behaviour is overloaded based upon the number and types of its
  arguments. The actual behaviour of a call of TypedArray depends upon
  the number and kind of arguments that are passed to it.
features: [BigInt]
---*/

assert.sameValue(typeof BigInt64Array, "function");
