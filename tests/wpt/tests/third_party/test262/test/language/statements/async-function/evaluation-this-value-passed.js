// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: pending
description: >
  The this value from the caller is present in the async function body
flags: [async]
---*/


async function foo(a) {
  assert.sameValue(this, a)
}
var obj = {};
foo.call(obj, obj).then($DONE, $DONE);

