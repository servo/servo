// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.tojson
description: >
  This value is coerced to an object.
info: |
  Date.prototype.toJSON ( key )

  1. Let O be ? ToObject(this value).
features: [Symbol]
---*/

var toJSON = Date.prototype.toJSON;
this.toISOString = function() { return 'global'; };

assert.throws(TypeError, function() {
  toJSON.call(undefined);
});

assert.throws(TypeError, function() {
  toJSON.call(null);
});

Number.prototype.toISOString = function() { return 'str'; };
assert.sameValue(toJSON.call(10), 'str');

Symbol.prototype.toISOString = function() { return 10; };
assert.sameValue(toJSON.call(Symbol()), 10);
