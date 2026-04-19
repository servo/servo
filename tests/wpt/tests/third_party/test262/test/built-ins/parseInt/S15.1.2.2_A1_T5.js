// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString
esid: sec-parseint-string-radix
description: Checking for Number object
---*/

assert.sameValue(parseInt(new Number(-1)), parseInt("-1"), 'parseInt(new Number(-1)) must return the same value returned by parseInt("-1")');

assert.sameValue(
  String(parseInt(new Number(Infinity))),
  "NaN",
  'String(parseInt(new Number(Infinity))) must return "NaN"'
);

assert.sameValue(String(parseInt(new Number(NaN))), "NaN", 'String(parseInt(new Number(NaN))) must return "NaN"');
