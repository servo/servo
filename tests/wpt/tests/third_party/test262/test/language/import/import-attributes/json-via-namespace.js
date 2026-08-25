// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-parse-json-module
description: May be imported via a module namespace object
flags: [module]
features: [import-attributes, json-modules]
---*/

import * as ns from './json-via-namespace_FIXTURE.json' with { type: 'json' };

assert.sameValue(Object.getOwnPropertyNames(ns).length, 1);
assert.sameValue(ns.default, 262);
