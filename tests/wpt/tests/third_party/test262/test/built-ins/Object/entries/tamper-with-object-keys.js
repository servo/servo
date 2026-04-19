// Copyright (C) 2015 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.entries
description: >
    Object.entries should not have its behavior impacted by modifications to Object.keys
author: Jordan Harband
---*/

function fakeObjectKeys() {
  throw new Test262Error('The overriden version of Object.keys was called!');
}

Object.keys = fakeObjectKeys;

assert.sameValue(Object.keys, fakeObjectKeys, 'Sanity check failed: could not modify the global Object.keys');
assert.sameValue(Object.entries({
  a: 1
}).length, 1, 'Expected object with 1 key to have 1 entry');
