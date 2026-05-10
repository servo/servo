// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-sync-function-prototype-properties
description: >
  %AsyncFunctionPrototype% has a [[Extensible]] of true
---*/

var AsyncFunction = async function foo() {}.constructor;
AsyncFunction.prototype.x = 1;
assert.sameValue(AsyncFunction.prototype.x, 1);
