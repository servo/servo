// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator remove leading StrWhiteSpaceChar
esid: sec-parseint-string-radix
description: "StrWhiteSpaceChar :: SP (U+0020)"
---*/

assert.sameValue(parseInt("\u00201"), parseInt("1"), 'parseInt("\\u00201") must return the same value returned by parseInt("1")');
assert.sameValue(parseInt("\u0020\u0020-1"), parseInt("-1"), 'parseInt("\\u0020\\u0020-1") must return the same value returned by parseInt("-1")');
assert.sameValue(parseInt(" 1"), parseInt("1"), 'parseInt(" 1") must return the same value returned by parseInt("1")');
assert.sameValue(parseInt("       1"), parseInt("1"), 'parseInt(" 1") must return the same value returned by parseInt("1")');

assert.sameValue(
  parseInt("       \u0020       \u0020-1"),
  parseInt("-1"),
  'parseInt(" \\u0020 \\u0020-1") must return the same value returned by parseInt("-1")'
);

//CHECK#6
assert.sameValue(parseInt("\u0020"), NaN, 'parseInt("\\u0020") must return NaN');
