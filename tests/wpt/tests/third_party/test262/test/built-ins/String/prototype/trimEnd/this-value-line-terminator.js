// Copyright (c) 2017 Valerie Young.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimend
description: TrimEnd removes all line terminators from the end of a string.
info: |
  Runtime Symantics: TrimString ( string, where )
  ...
  4. Else if where is "end", let T be a String value that is a copy of S with
     trailing white space removed.
  ...

  The definition of white space is the union of WhiteSpace and LineTerminator.

features: [string-trimming, String.prototype.trimEnd]
---*/

var trimEnd = String.prototype.trimEnd;

// A string of all valid LineTerminator Unicode code points
var lt = '\u000A\u000D\u2028\u2029';

var str = lt + 'a' + lt + 'b' + lt;
var expected = lt + 'a' + lt + 'b';

assert.sameValue(
  trimEnd.call(str),
  expected
);
