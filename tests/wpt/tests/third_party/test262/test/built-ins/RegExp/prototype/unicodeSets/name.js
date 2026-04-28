// Copyright (C) 2022 Mathias Bynens, Ron Buckton, and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-regexp.prototype.unicodesets
description: >
  RegExp.prototype.unicodeSets name
info: |
  17 ECMAScript Standard Built-in Objects

  Functions that are specified as get or set accessor functions of built-in
  properties have "get " or "set " prepended to the property name string.
includes: [propertyHelper.js]
features: [regexp-v-flag]
---*/

var desc = Object.getOwnPropertyDescriptor(RegExp.prototype, "unicodeSets");

assert.sameValue(
  desc.get.name,
  "get unicodeSets"
);

verifyProperty(desc.get, "name", {
  enumerable: false,
  writable: false,
  configurable: true,
});
