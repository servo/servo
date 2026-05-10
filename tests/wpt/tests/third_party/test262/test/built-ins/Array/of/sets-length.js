// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.of
description: >
  Calls the length setter if available
info: |
  Array.of ( ...items )

  ...
  9. Let setStatus be Set(A, "length", len, true).
  ...
---*/

var hits = 0;
var value;
var _this_;

function Pack() {
  Object.defineProperty(this, "length", {
    set: function(len) {
      hits++;
      value = len;
      _this_ = this;
    }
  });
}

var result = Array.of.call(Pack, 'wolves', 'cards', 'cigarettes', 'lies');

assert.sameValue(hits, 1, 'The value of hits is expected to be 1');
assert.sameValue(
  value, 4,
  'The value of value is expected to be 4'
);
assert.sameValue(_this_, result, 'The value of _this_ is expected to equal the value of result');
