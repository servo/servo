// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-async-function-instances
description: >
  Async function instances do not have a prototype property.
---*/
async function foo() {};
assert.sameValue(foo.prototype, undefined, 'foo.prototype should be undefined');
assert(!foo.hasOwnProperty('prototype'), 'foo.prototype should not exist');
