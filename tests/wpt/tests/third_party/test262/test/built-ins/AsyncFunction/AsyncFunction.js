// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: sec-async-function-objects
description: >
  %AsyncFunction% exists and is a function
---*/

var AsyncFunction = async function foo() {}.constructor;
assert.sameValue(typeof AsyncFunction, "function");
