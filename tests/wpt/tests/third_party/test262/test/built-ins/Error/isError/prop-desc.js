// Copyright (C) 2024 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.iserror
description: Property descriptor for Error.isError
info: |
  Every other data property described in clauses 18 through 26 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.
includes: [propertyHelper.js]
features: [Error.isError]
---*/

verifyProperty(Error, "isError", {
  writable: true,
  enumerable: false,
  configurable: true
});
