// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  compareArray correctly formats Symbols in error message.
includes: [compareArray.js]
features: [Symbol]
---*/

var threw = false;

try {
  assert.compareArray([Symbol()], [Symbol('desc')]);
} catch (err) {
  threw = true;

  assert.sameValue(err.constructor, Test262Error);
  assert(err.message.indexOf('[Symbol()]') !== -1);
  assert(err.message.indexOf('[Symbol(desc)]') !== -1);
}

assert(threw);
