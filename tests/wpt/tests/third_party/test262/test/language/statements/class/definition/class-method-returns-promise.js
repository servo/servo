// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: pending
features: [async-functions]
description: >
  Async function expressions return promises
---*/
class Foo {
  async method() {};
}
var p = new Foo().method();
assert(p instanceof Promise, "async functions return promise instances");
