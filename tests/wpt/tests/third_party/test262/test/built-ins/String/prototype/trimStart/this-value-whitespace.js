// Copyright (c) 2017 Valerie Young.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimstart
description: TrimStart removes all whitespace from the start of a string.
info: |
  Runtime Symantics: TrimString ( string, where )
  ...
  3. If where is "start", let T be a String value that is a copy of S with
     trailing white space removed.
  ...

  The definition of white space is the union of WhiteSpace and LineTerminator.
  When determining whether a Unicode code point is in Unicode general category
  “Zs”, code unit sequences are interpreted as UTF-16 encoded code point
  sequences as specified in 6.1.4.

features: [string-trimming, String.prototype.trimStart]
---*/

var trimStart = String.prototype.trimStart;

// A string of all valid WhiteSpace Unicode code points
var wspc = '\x09\x0A\x0B\x0C\x0D\x20\xA0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A\u202F\u205F\u3000\u2028\u2029\uFEFF';

var str = wspc + 'a' + wspc + 'b' + wspc;
var expected = 'a' + wspc + 'b' + wspc;

assert.sameValue(
  trimStart.call(str),
  expected
);
