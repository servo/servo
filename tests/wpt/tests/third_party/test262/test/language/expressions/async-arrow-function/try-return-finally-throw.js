// Copyright 2017 Caitlin Potter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Caitlin Potter <caitp@igalia.com>
esid: pending
description: >
  Implementations must defer resolving an async function's Promise until after
  all finally blocks have been evaluated.
flags: [async]
---*/

var f = async() => {
  try {
    return "early-return";
  } finally {
    throw "override";
  }
};

f().then($DONE, function(value) {
  assert.sameValue(value, "override", "Exception thrown in finally block");
}).then($DONE, $DONE);
