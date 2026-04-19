// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator remove leading StrWhiteSpaceChar
esid: sec-parseint-string-radix
description: "StrWhiteSpaceChar :: TAB (U+0009)"
---*/

assert.sameValue(parseInt("\u00091"), parseInt("1"), 'parseInt("\\u00091") must return the same value returned by parseInt("1")');

assert.sameValue(parseInt("\u0009\u0009-1"), parseInt("-1"), 'parseInt("\\u0009\\u0009-1") must return the same value returned by parseInt("-1")');

assert.sameValue(parseInt("	1"), parseInt("1"), 'parseInt(" 1") must return the same value returned by parseInt("1")');

assert.sameValue(parseInt("			1"), parseInt("1"), 'parseInt(" 1") must return the same value returned by parseInt("1")');

assert.sameValue(
  parseInt("			\u0009			\u0009-1"),
  parseInt("-1"),
  'parseInt(" \\u0009 \\u0009-1") must return the same value returned by parseInt("-1")'
);

assert.sameValue(parseInt("\u0009"), NaN, 'parseInt("\\u0009") must return NaN');
