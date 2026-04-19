// Copyright (C) 2016 the V8 project authors. All rights reserved.
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

  [...] the internal procedure that would result from evaluating S as a
  Pattern[~U] (Pattern[+U] if F contains "u") must behave identically to the
  internal procedure given by the constructed object's [[RegExpMatcher]]
  internal slot.
---*/

var re = eval('/' + /ab{2,4}c$/.source + '/');

assert(re.test('abbc'), 'input: abbc');
assert(re.test('abbbc'), 'input: abbbc');
assert(re.test('abbbbc'), 'input: abbbbc');
assert(re.test('xabbc'), 'input: xabbc');
assert(re.test('xabbbc'), 'input: xabbbc');
assert(re.test('xabbbbc'), 'input: xabbbbc');

assert.sameValue(re.test('ac'), false, 'input: ac');
assert.sameValue(re.test('abc'), false, 'input: abc');
assert.sameValue(re.test('abbcx'), false, 'input: abbcx');
assert.sameValue(re.test('bbc'), false, 'input: bbc');
assert.sameValue(re.test('abb'), false, 'input: abb');
assert.sameValue(re.test('abbbbbc'), false, 'input: abbbbbc');
