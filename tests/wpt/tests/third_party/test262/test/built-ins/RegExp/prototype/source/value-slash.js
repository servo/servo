// Copyright (C) 2016 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.source
description: Return value can be used to create an equivalent RegExp
info: |
  [...]
  5. Let src be R.[[OriginalSource]].
  6. Let flags be R.[[OriginalFlags]].
  7. Return EscapeRegExpPattern(src, flags).

  21.2.3.2.4 Runtime Semantics: EscapeRegExpPattern

  [...] The code points / or any LineTerminator occurring in the pattern
  shall be escaped in S as necessary to ensure that the String value
  formed by concatenating the Strings "/", S, "/", and F can be parsed
  (in an appropriate lexical context) as a RegularExpressionLiteral that
  behaves identically to the constructed regular expression.
---*/

var re = eval('/' + new RegExp('/').source + '/');

assert(re.test('/'), 'input: "/"');
assert(re.test('_/_'), 'input: "_/_"');
assert(!re.test('\\'), 'input: "\\"');
