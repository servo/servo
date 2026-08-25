// Copyright 2015 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
 Symbol.species is a well-known symbol
es6id: 19.4.2.10
author: Sam Mikes
description: Symbol.species exists
includes: [propertyHelper.js]
features: [Symbol.species]
---*/

assert(Symbol !== undefined, "Symbol exists");
assert(Symbol.species !== undefined, "Symbol.species exists");

verifyProperty(Symbol, "species", {
  writable: false,
  enumerable: false,
  configurable: false,
});
