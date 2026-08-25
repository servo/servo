// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Uses [[DefineOwnProperty]] rather than [[Set]].
esid: sec-object.fromentries
features: [Object.fromEntries]
---*/

Object.defineProperty(Object.prototype, 'property', {
  get: function() {
    throw new Test262Error('should not trigger getter on Object.prototype');
  },
  set: function() {
    throw new Test262Error('should not trigger setter on Object.prototype');
  },
});

var result = Object.fromEntries([['property', 'value']]);
assert.sameValue(result['property'], 'value', '');
