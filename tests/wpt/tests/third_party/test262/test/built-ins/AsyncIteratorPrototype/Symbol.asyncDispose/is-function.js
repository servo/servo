// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%asynciteratorprototype%-@@asyncDispose
description: >
  AsyncIterator.prototype[@@asyncDispose] is a built-in function
features: [explicit-resource-management]
---*/

async function* generator() {}
const AsyncIteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf(generator.prototype))

assert.sameValue(typeof AsyncIteratorPrototype[Symbol.asyncDispose], 'function');
