// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects-isextensible
description: The [[IsExtensible]] internal method returns `false`
flags: [module]
---*/

import * as ns from './is-extensible.js';

assert.sameValue(Object.isExtensible(ns), false);
