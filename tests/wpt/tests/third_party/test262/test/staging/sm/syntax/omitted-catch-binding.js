// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var state = 'initial';
try {
    throw new Error('caught');
    state = 'unreachable';
} catch { // Note the lack of a binding
    assert.sameValue(state, 'initial');
    state = 'caught';
}
assert.sameValue(state, 'caught');


var sigil1 = {};
try {
  throw sigil1;
} catch (e) {
  assert.sameValue(e, sigil1);
}


var sigil2 = {};
var reached = false;
try {
  try {
    throw sigil1;
  } catch {
    reached = true;
  } finally {
    throw sigil2;
  }
} catch (e) {
  assert.sameValue(e, sigil2);
}
assert.sameValue(reached, true);

