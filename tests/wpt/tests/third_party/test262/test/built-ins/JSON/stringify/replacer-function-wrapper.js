// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Wrapper is plain extensible object with single data property.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  9. Let wrapper be ObjectCreate(%ObjectPrototype%).
  10. Let status be CreateDataProperty(wrapper, the empty String, value).
includes: [propertyHelper.js]
---*/

Object.defineProperty(Object.prototype, '', {
  set: function() {
    throw new Test262Error('[[Set]] should not be called.');
  },
});

var value = {};
var wrapper;
JSON.stringify(value, function() {
  wrapper = this;
});

assert.sameValue(typeof wrapper, 'object');
assert.sameValue(Object.getPrototypeOf(wrapper), Object.prototype);
assert.sameValue(Object.getOwnPropertyNames(wrapper).length, 1);
assert(Object.isExtensible(wrapper));

verifyProperty(wrapper, '', {
  value: value,
  writable: true,
  enumerable: true,
  configurable: true,
});
