// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Testing descriptor property of Math.atanh
includes: [propertyHelper.js]
es6id: 20.2.2.7
---*/

verifyProperty(Math, "atanh", {
  writable: true,
  enumerable: false,
  configurable: true
});
