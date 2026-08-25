// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [propertyHelper.js]
description: |
  pending
esid: pending
---*/

verifyProperty(Function.prototype, "length",
    {value: 0, writable: false, enumerable: false, configurable: true});

assert.sameValue(Function.prototype.prototype, undefined);
assert.sameValue(Function.prototype.callee, undefined);
assert.throws(TypeError, () => Function.prototype.caller);
assert.throws(TypeError, () => Function.prototype.arguments);
