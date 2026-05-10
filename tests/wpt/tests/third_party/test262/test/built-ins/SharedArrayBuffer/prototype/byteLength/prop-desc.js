// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  "byteLength" property of SharedArrayBuffer.prototype
includes: [propertyHelper.js]
features: [SharedArrayBuffer]
---*/

var desc = Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype, "byteLength");

assert.sameValue(desc.set, undefined);
assert.sameValue(typeof desc.get, "function");

verifyNotEnumerable(SharedArrayBuffer.prototype, "byteLength");
verifyConfigurable(SharedArrayBuffer.prototype, "byteLength");
