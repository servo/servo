// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-parse-json-module
description: Correctly parses the JSON representation of an array
info: |
  # 1.4 ParseJSONModule ( source )

  The abstract operation ParseJSONModule takes a single argument source which
  is a String representing the contents of a module.

  1. Let json be ? Call(%JSON.parse%, undefined, « source »).
  2. Return CreateDefaultExportSyntheticModule(json).

  To more fully verify parsing correctness, the source text of the imported
  module record includes non-printable characters (specifically, all four forms
  of JSON's so-called "whitespace" token) both before and after the "value."
flags: [module]
features: [import-attributes, json-modules]
---*/

import value from './json-value-array_FIXTURE.json' with { type: 'json' };

assert(Array.isArray(value), 'the exported value is an array');
assert.sameValue(
  Object.getPrototypeOf(value),
  Array.prototype,
  'the exported value is not a subclass of Array'
);
assert.sameValue(Object.getOwnPropertyNames(value).length, 7);
assert.sameValue(value.length, 6);

assert.sameValue(value[0], -1.2345);
assert.sameValue(value[1], true);
assert.sameValue(value[2], 'a string value');
assert.sameValue(value[3], null);

assert.sameValue(Object.getPrototypeOf(value[4]), Object.prototype);
assert.sameValue(Object.getOwnPropertyNames(value[4]).length, 0);

assert(Array.isArray(value[5]), 'the fifth element is an array');
assert.sameValue(
  Object.getPrototypeOf(value[5]),
  Array.prototype,
  'the fifth element is not a subclass of Array'
);
assert.sameValue(Object.getOwnPropertyNames(value[5]).length, 1);
