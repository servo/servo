// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: pending
description: >
  The return value of the async function resolves the promise
flags: [async]
---*/

async function foo() {
  return 42;
}

foo().then(function (v) {
  assert.sameValue(v, 42);
  $DONE();
}, $DONE);
