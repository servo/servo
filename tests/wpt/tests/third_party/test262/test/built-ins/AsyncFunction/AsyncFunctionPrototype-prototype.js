// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-sync-function-prototype-properties
description: AsyncFunction.prototype has a [[prototype]] of Function.prototype
---*/
var AsyncFunction = async function foo() {}.constructor;
assert.sameValue(Object.getPrototypeOf(AsyncFunction.prototype), Function.prototype);
