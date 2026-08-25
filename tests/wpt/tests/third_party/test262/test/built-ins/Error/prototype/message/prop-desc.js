// Copyright (c) 2021 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-error.prototype.message
description: Property descriptor of Error.prototype.message
includes: [propertyHelper.js]
---*/

verifyProperty(Error.prototype, 'message', {
  enumerable: false,
  writable: true,
  configurable: true,
  value: ''
});
