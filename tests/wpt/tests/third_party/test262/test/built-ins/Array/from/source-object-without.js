// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Source is an object without length property
esid: sec-array.from
es6id: 22.1.2.1
---*/

var obj = {
  0: 2,
  1: 4,
  2: 8,
  3: 16
}

var a = Array.from(obj);
assert.sameValue(a.length, 0, 'The value of a.length is expected to be 0');
