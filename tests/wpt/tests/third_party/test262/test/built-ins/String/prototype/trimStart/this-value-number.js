// Copyright (c) 2017 Valerie Young.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimstart
description: Behavoir when "this" value is a number.
info: |
  Runtime Semantics: TrimString ( string, where )
  2. Let S be ? ToString(str).

  ToString ( argument )
  Argument Type: Number
  Result: NumberToString(argument)
features: [string-trimming, String.prototype.trimStart]
---*/

var trimStart = String.prototype.trimStart

assert.sameValue(
  trimStart.call(NaN),
  'NaN',
  'String.prototype.trimStart.call(NaN)'
);

assert.sameValue(
  trimStart.call(Infinity),
  'Infinity',
  'String.prototype.trimStart.call(Infinity)'
);

assert.sameValue(
  trimStart.call(-0),
  '0',
  'String.prototype.trimStart.call(-0)'
);

assert.sameValue(
  trimStart.call(1),
  '1',
  'String.prototype.trimStart.call(1)'
);

assert.sameValue(
  trimStart.call(-1),
  '-1',
  'String.prototype.trimStart.call(-1)'
);
