// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  When composing a message, errors from ToString are handled.
features: [async-functions]
---*/

var threw = false;
var asyncFunProto = Object.getPrototypeOf(async function() {});

try {
  assert(asyncFunProto);
} catch (err) {
  threw = true;
  if (err.constructor !== Test262Error) {
    throw new Error('Expected a Test262Error, but a "' + err.constructor.name + '" was thrown.');
  }
}

if (!threw) {
  throw new Error('Expected a Test262Error, but no error was thrown.');
}
