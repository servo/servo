// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-setprototypeof-v
description: >
  The [[SetPrototypeOf]] internal method returns `true` if
  passed `null`
flags: [module]
---*/

import * as ns from './set-prototype-of-null.js';

assert.sameValue(typeof Object.setPrototypeOf, 'function');
assert.sameValue(ns, Object.setPrototypeOf(ns, null));
