// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If Z is empty, return NaN
esid: sec-parseint-string-radix
description: x is not a radix-R digit
---*/

assert.sameValue(parseInt("$0x"), NaN, 'parseInt("$0x") must return NaN');
assert.sameValue(parseInt("$0X"), NaN, 'parseInt("$0X") must return NaN');
assert.sameValue(parseInt("$$$"), NaN, 'parseInt("$$$") must return NaN');
assert.sameValue(parseInt(""), NaN, 'parseInt("") must return NaN');
assert.sameValue(parseInt(" "), NaN, 'parseInt(" ") must return NaN');
