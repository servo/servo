// Copyright (C) 2017 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  verifyProperty allows restoring the original accessor descriptor
includes: [propertyHelper.js]
features: [Symbol]
---*/

var obj;
var prop = Symbol(1);
var desc = { enumerable: true, configurable: true, get() { return 42; }, set() {} };

obj = {};
Object.defineProperty(obj, prop, desc);

verifyProperty(obj, prop, desc);

assert.sameValue(
  Object.prototype.hasOwnProperty.call(obj, prop),
  false
);

obj = {};
Object.defineProperty(obj, prop, desc);

verifyProperty(obj, prop, desc, { restore: true });

assert.sameValue(
  Object.prototype.hasOwnProperty.call(obj, prop),
  true
);
assert.sameValue(obj[prop], 42);
assert.sameValue(
  Object.getOwnPropertyDescriptor(obj, prop).get,
  desc.get
);

assert.sameValue(
  Object.getOwnPropertyDescriptor(obj, prop).set,
  desc.set
);
