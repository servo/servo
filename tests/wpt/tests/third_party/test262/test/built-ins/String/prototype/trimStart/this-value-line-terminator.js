// Copyright (c) 2017 Valerie Young.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimstart
description: TrimStart removes all line terminators from the start of a string.
info: |
  Runtime Symantics: TrimString ( string, where )
  ...
  4. If where is "start", let T be a String value that is a copy of S with
     trailing white space removed.
  ...

  The definition of white space is the union of WhiteSpace and LineTerminator.

features: [string-trimming, String.prototype.trimStart]
---*/

var trimStart = String.prototype.trimStart;

// A string of all valid LineTerminator Unicode code points
var lt = '\u000A\u000D\u2028\u2029';

var str = lt + 'a' + lt + 'b' + lt;
var expected = 'a' + lt + 'b' + lt;

assert.sameValue(
  trimStart.call(str),
  expected
);
