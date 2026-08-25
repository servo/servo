// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-parse-json-module
description: Correctly parses the JSON representation of null
info: |
  # 1.4 ParseJSONModule ( source )

  The abstract operation ParseJSONModule takes a single argument source which
  is a String representing the contents of a module.

  1. Let json be ? Call(%JSON.parse%, undefined, « source »).
  2. Return CreateDefaultExportSyntheticModule(json).
flags: [module]
features: [import-attributes, json-modules]
---*/

import value from './json-value-null_FIXTURE.json' with { type: 'json' };

assert.sameValue(value, null);
