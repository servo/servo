// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Testing descriptor property of Math.log1p
includes: [propertyHelper.js]
es6id: 20.2.2.21
---*/

verifyProperty(Math, "log1p", {
  writable: true,
  enumerable: false,
  configurable: true,
});
