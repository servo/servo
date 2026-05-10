// Copyright (C) 2016 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-@@iterator
description: Module namespace objects lack a Symbol.iterator property.
flags: [module]
features: [Symbol.iterator]
---*/

import * as ns from './Symbol.iterator.js';

assert.sameValue(Object.prototype.hasOwnProperty.call(ns, Symbol.iterator), false);
