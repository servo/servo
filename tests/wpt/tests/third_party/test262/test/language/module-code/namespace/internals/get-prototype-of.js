// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-getprototypeof
description: The [[GetPrototypeOf]] internal method returns `null`
flags: [module]
---*/

import * as ns from './get-prototype-of.js';

assert.sameValue(ns instanceof Object, false);
assert.sameValue(Object.getPrototypeOf(ns), null);
