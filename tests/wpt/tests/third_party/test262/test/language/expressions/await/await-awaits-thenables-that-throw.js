// Copyright 2016 Microsoft, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Brian Terlson <brian.terlson@microsoft.com>
esid: pending
description: >
  Await can await any thenable.
flags: [async]
includes: [asyncHelpers.js]
---*/

var error = {};
var thenable = {
  then: function (resolve, reject) {
    throw error;
  }
}
async function foo() {
  var caught = false;
  try {
    await thenable;
  } catch(e) {
    caught = true;
    assert.sameValue(e, error);
  }

  assert(caught);
}

asyncTest(foo);

