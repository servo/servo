// Copyright (C) 2024 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-json.rawjson
description: >
  Property type and descriptor.
info: |
  JSON.rawJSON ( text )

  18 ECMAScript Standard Built-in Objects
  ...
  Every other data property described in clauses 19 through 28 and in Annex B.2
  has the attributes { [[Writable]]: true, [[Enumerable]]: false,
  [[Configurable]]: true } unless otherwise specified.

includes: [propertyHelper.js]
features: [json-parse-with-source]
---*/

verifyProperty(JSON, 'rawJSON', {
  enumerable: false,
  writable: true,
  configurable: true
});
