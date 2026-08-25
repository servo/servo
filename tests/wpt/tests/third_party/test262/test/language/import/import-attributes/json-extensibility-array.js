// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-parse-json-module
description: Creates extensible arrays
flags: [module]
includes: [propertyHelper.js]
features: [import-attributes, json-modules]
---*/

import value from './json-value-array_FIXTURE.json' with { type: 'json' };

value.test262property = 'test262 value';

verifyProperty(value, 'test262property', {
  value: 'test262 value',
  writable: true,
  enumerable: true,
  configurable: true
});
