// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.at
description: >
  Creates an iterator from a custom object.
info: |
  String.prototype.at( index )

  If k < 0 or k ≥ len, then return undefined.
features: [String.prototype.at]
---*/
assert.sameValue(typeof String.prototype.at, 'function');

let s = "";

assert.sameValue(s.at(-2), undefined, 's.at(-2) must return undefined'); // wrap around the end
assert.sameValue(s.at(0), undefined, 's.at(0) must return undefined');
assert.sameValue(s.at(1), undefined, 's.at(1) must return undefined');

