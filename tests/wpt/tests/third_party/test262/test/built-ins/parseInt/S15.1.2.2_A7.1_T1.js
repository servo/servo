// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If Z is empty, return NaN
esid: sec-parseint-string-radix
description: Complex test. R in [2, 36]
---*/

//CHECK#
for (var i = 2; i <= 36; i++) {
  assert.sameValue(parseInt("$string", i), NaN, 'parseInt("$string", i) must return NaN');
}
