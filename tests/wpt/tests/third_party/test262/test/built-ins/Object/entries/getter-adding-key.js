// Copyright (C) 2015 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.entries
description: Object.entries does not see a new element added by a getter that is hit during iteration
author: Jordan Harband
---*/

var bAddsC = {
  a: 'A',
  get b() {
    this.c = 'C';
    return 'B';
  }
};

var result = Object.entries(bAddsC);

assert.sameValue(Array.isArray(result), true, 'result is an array');
assert.sameValue(result.length, 2, 'result has 2 items');

assert.sameValue(Array.isArray(result[0]), true, 'first entry is an array');
assert.sameValue(Array.isArray(result[1]), true, 'second entry is an array');

assert.sameValue(result[0][0], 'a', 'first entry has key "a"');
assert.sameValue(result[0][1], 'A', 'first entry has value "A"');
assert.sameValue(result[1][0], 'b', 'second entry has key "b"');
assert.sameValue(result[1][1], 'B', 'second entry has value "B"');
