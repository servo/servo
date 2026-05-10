// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: pending
description: >
  Await can await any thenable. If the thenable's then is not callable,
  await evaluates to the thenable
flags: [async]
includes: [asyncHelpers.js]
---*/

async function foo() {
  var thenable = { then: 42 };
  var res = await thenable;
  assert.sameValue(res, thenable);
}

asyncTest(foo);

