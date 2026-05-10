// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.parse
description: >
  Wrapper is plain extensible object with single data property.
info: |
  JSON.parse ( text [ , reviver ] )

  [...]
  7. If IsCallable(reviver) is true, then
    a. Let root be ObjectCreate(%Object.prototype%).
    b. Let rootName be the empty String.
    c. Perform ! CreateDataPropertyOrThrow(root, rootName, unfiltered).
includes: [propertyHelper.js]
---*/

Object.defineProperty(Object.prototype, '', {
  set: function() {
    throw new Test262Error('[[Set]] should not be called.');
  },
});

var wrapper;
JSON.parse('2', function() {
  wrapper = this;
});

assert.sameValue(typeof wrapper, 'object');
assert.sameValue(Object.getPrototypeOf(wrapper), Object.prototype);
assert.sameValue(Object.getOwnPropertyNames(wrapper).length, 1);
assert(Object.isExtensible(wrapper));

verifyProperty(wrapper, '', {
  value: 2,
  writable: true,
  enumerable: true,
  configurable: true,
});
