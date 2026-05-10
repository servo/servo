// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.at
description: >
  Returns the item value at the specified index
info: |
  String.prototype.at ( )

  Let O be ? ToObject(this value).
  Let len be ? LengthOfStringLike(O).
  Let relativeIndex be ? ToInteger(index).
  If relativeIndex ≥ 0, then
    Let k be relativeIndex.
  Else,
    Let k be len + relativeIndex.
  If k < 0 or k ≥ len, then return undefined.
  Return ? Get(O, ! ToString(k)).

features: [String.prototype.at]
---*/
assert.sameValue(typeof String.prototype.at, 'function');

let s = "12345";

assert.sameValue(s.at(0), "1", 's.at(0) must return "1"');
assert.sameValue(s.at(1), "2", 's.at(1) must return "2"');
assert.sameValue(s.at(2), "3", 's.at(2) must return "3"');
assert.sameValue(s.at(3), "4", 's.at(3) must return "4"');
assert.sameValue(s.at(4), "5", 's.at(4) must return "5"');
