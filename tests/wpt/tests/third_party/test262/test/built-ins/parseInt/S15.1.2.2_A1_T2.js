// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString
esid: sec-parseint-string-radix
description: Checking for number primitive
---*/

assert.sameValue(parseInt(-1), parseInt("-1"), 'parseInt(-1) must return the same value returned by parseInt("-1")');
assert.sameValue(String(parseInt(Infinity)), "NaN", 'String(parseInt(Infinity)) must return "NaN"');
assert.sameValue(String(parseInt(NaN)), "NaN", 'String(parseInt(NaN)) must return "NaN"');
assert.sameValue(parseInt(-0), 0, 'parseInt(-0) must return 0');
