// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: pending
description: >
  Unmapped arguments object is used when the async function doesn't have a
  simple parameter list.
flags: [async]
---*/


async function foo(a = 42) {
  arguments[0] = 2;
  assert.sameValue(a, 1);

  a = 3;
  assert.sameValue(arguments[0], 2);
}

foo(1).then($DONE, $DONE);

