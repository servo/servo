// Copyright (C) 2017 Mozilla Corporation.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics-@@tostringtag
description: >
    `Symbol.toStringTag` property descriptor on Atomics
info: |
    The initial value of the @@toStringTag property is the String value
    "Atomics".

    This property has the attributes { [[Writable]]: false, [[Enumerable]]:
    false, [[Configurable]]: true }.
includes: [propertyHelper.js]
features: [Atomics, Symbol, Symbol.toStringTag]
---*/

assert.sameValue(
  Atomics[Symbol.toStringTag],
  'Atomics',
  'The value of Atomics[Symbol.toStringTag] is "Atomics"'
);

verifyProperty(Atomics, Symbol.toStringTag, {
  writable: false,
  enumerable: false,
  configurable: true,
});
