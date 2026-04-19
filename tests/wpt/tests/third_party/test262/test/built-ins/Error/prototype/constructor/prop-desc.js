// Copyright (c) 2021 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-error.prototype.constructor
description: Property descriptor of Error.prototype.constructor
includes: [propertyHelper.js]
---*/

verifyProperty(Error.prototype, 'constructor', {
  enumerable: false,
  writable: true,
  configurable: true,
  value: Error
});
