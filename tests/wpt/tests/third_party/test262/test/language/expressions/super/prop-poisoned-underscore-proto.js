// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-SuperProperty
description: >
  SuperProperty should directly call [[GetPrototypeOf]] internal method.
info: |
  MakeSuperPropertyReference ( actualThis, propertyKey, strict )

  [...]
  3. Let baseValue be ? env.GetSuperBase().

  GetSuperBase ( )

  [...]
  5. Return ? home.[[GetPrototypeOf]]().
---*/

Object.defineProperty(Object.prototype, '__proto__', {
  get: function() {
    throw new Test262Error('should not be called');
  },
});

var obj = {
  superExpression() {
    return super['CONSTRUCTOR'.toLowerCase()];
  },
  superIdentifierName() {
    return super.toString();
  },
};

assert.sameValue(obj.superExpression(), Object);
assert.sameValue(obj.superIdentifierName(), '[object Object]');
