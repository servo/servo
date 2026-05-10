// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.at
description: >
  Property type and descriptor.
info: |
  String.prototype.at( index )

  Let relativeIndex be ? ToInteger(index).

features: [String.prototype.at]
---*/
assert.sameValue(typeof String.prototype.at, 'function');

let s = "01";

assert.sameValue(s.at(false), '0', 's.at(false) must return 0');
assert.sameValue(s.at(null), '0', 's.at(null) must return 0');
assert.sameValue(s.at(undefined), '0', 's.at(undefined) must return 0');
assert.sameValue(s.at(""), '0', 's.at("") must return 0');
assert.sameValue(s.at(function() {}), '0', 's.at(function() {}) must return 0');
assert.sameValue(s.at([]), '0', 's.at([]) must return 0');

assert.sameValue(s.at(true), '1', 's.at(true) must return 1');
assert.sameValue(s.at("1"), '1', 's.at("1") must return 1');
