// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Testing descriptor property of Math.acosh
includes: [propertyHelper.js]
es6id: 20.2.2.3
---*/

verifyProperty(Math, "acosh", {
  writable: true,
  enumerable: false,
  configurable: true
});
