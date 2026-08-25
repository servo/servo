// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
info: Operator use ToNumber
esid: sec-parseint-string-radix
description: Checking for Number object
---*/

assert.sameValue(
  parseInt("11", new Number(2)),
  parseInt("11", 2),
  'parseInt("11", new Number(2)) must return the same value returned by parseInt("11", 2)'
);

assert.sameValue(
  parseInt("11", new Number(Infinity)),
  parseInt("11", Infinity),
  'parseInt("11", new Number(Infinity)) must return the same value returned by parseInt("11", Infinity)'
);
