// Copyright (c) 2017 Valerie Young.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimend
description: Behavoir when "this" value is a number.
info: |
  Runtime Semantics: TrimString ( string, where )
  2. Let S be ? ToString(str).

  ToString ( argument )
  Argument Type: Number
  Result: NumberToString(argument)
features: [string-trimming, String.prototype.trimEnd]
---*/

var trimEnd = String.prototype.trimEnd

assert.sameValue(
  trimEnd.call(NaN),
  'NaN',
  'String.prototype.trimEnd.call(NaN)'
);

assert.sameValue(
  trimEnd.call(Infinity),
  'Infinity',
  'String.prototype.trimEnd.call(Infinity)'
);

assert.sameValue(
  trimEnd.call(-0),
  '0',
  'String.prototype.trimEnd.call(-0)'
);

assert.sameValue(
  trimEnd.call(1),
  '1',
  'String.prototype.trimEnd.call(1)'
);

assert.sameValue(
  trimEnd.call(-1),
  '-1',
  'String.prototype.trimEnd.call(-1)'
);
