// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-regexp.prototype.source
description: >
  Return value can be used to create an equivalent RegExp when the
  [[OriginalFlags]] internal slot contains the `u` flag
info: |
  [...]
  5. Let src be R.[[OriginalSource]].
  6. Let flags be R.[[OriginalFlags]].
  7. Return EscapeRegExpPattern(src, flags).

  21.2.3.2.4 Runtime Semantics: EscapeRegExpPattern

  [...] the internal procedure that would result from evaluating S as a
  Pattern[~U] (Pattern[+U] if F contains "u") must behave identically to the
  internal procedure given by the constructed object's [[RegExpMatcher]]
  internal slot.
---*/

var re;

re = eval('/' + /\ud834\udf06/u.source + '/u');

assert.sameValue(re.test('\ud834\udf06'), true);
assert.sameValue(re.test('𝌆'), true);

re = eval('/' + /\u{1d306}/u.source + '/u');

assert.sameValue(re.test('\ud834\udf06'), true);
assert.sameValue(re.test('𝌆'), true);
