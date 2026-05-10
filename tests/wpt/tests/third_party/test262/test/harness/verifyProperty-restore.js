// Copyright (C) 2017 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  verifyProperty allows restoring the original descriptor
includes: [propertyHelper.js]
---*/

var obj;
var prop = 'prop';
var desc = { enumerable: true, configurable: true, writable: true, value: 42 };

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
