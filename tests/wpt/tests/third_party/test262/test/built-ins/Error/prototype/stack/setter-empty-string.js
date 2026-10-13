// Copyright (C) 2026 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-set-error.prototype.stack
description: >
  An empty string is a String, so the setter accepts it.
info: |
  set Error.prototype.stack

  1. Let E be the this value.
  2. If E is not an Object, throw a TypeError exception.
  3. If v is not a String, throw a TypeError exception.
  4. Perform ? SetterThatIgnoresPrototypeProperties(this value, %Error.prototype%, "stack", v).
  5. Return undefined.
includes: [propertyHelper.js, nativeErrors.js]
features: [error-stack-accessor]
---*/

var set = Object.getOwnPropertyDescriptor(Error.prototype, 'stack').set;

for (var i = 0; i < nativeErrors.length; ++i) {
  var Ctor = nativeErrors[i];
  var err = new Ctor('msg');
  set.call(err, '');

  // Verify the round-trip via property access before invoking verifyProperty
  // (which deletes the configurable own property as part of its check).
  assert.sameValue(err.stack, '', Ctor.name + ': empty string round-trips through property access');

  verifyProperty(err, 'stack', {
    value: '',
    writable: true,
    enumerable: true,
    configurable: true,
  });
}
