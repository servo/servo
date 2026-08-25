// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Behavior when regexp is null
info: |
  String.prototype.matchAll ( regexp )
    1. Let O be ? RequireObjectCoercible(this value).
    2. If regexp is neither undefined nor null, then
      [...]
    3. Return ? MatchAllIterator(regexp, O).

  MatchAllIterator( regexp, O )
    [...]
    2. If ? IsRegExp(regexp) is true, then
      [...]
    3. Else,
      a. Let R be RegExpCreate(regexp, "g").
features: [String.prototype.matchAll]
includes: [compareArray.js, compareIterator.js, regExpUtils.js]
---*/

var str = 'a';

assert.compareIterator(str.matchAll(undefined), [
  matchValidator([''], 0, str),
  matchValidator([''], 1, str)
]);
