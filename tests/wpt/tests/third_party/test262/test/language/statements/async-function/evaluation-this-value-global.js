// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: pending
description: >
  The this value is set to the global object when not passed in sloppy mode.
flags: [noStrict, async]
includes: [asyncHelpers.js]
---*/

var glob = this;
async function foo() {
  assert.sameValue(this, glob);
}

asyncTest(foo);
