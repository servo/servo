// Copyright (C) 2025 Mozilla Foundation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-create-text-module
description: May be imported via a module namespace object
flags: [module]
features: [import-attributes, import-text]
---*/

import * as ns from './text-via-namespace_FIXTURE' with { type: 'text' };

assert.sameValue(Object.getOwnPropertyNames(ns).length, 1);
assert.sameValue(typeof ns.default, 'string');
