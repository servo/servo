// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    outermost binding updated in catch block; nested block let declaration unseen outside of block
---*/
var caught = false;
try {
  {
    let xx = 18;
    throw 25;
  }
} catch (e) {
  caught = true;
  assert.sameValue(e, 25);
  (function () {
    try {
      // NOTE: This checks that the block scope containing xx has been
      // removed from the context chain.
      assert.sameValue(xx, undefined);
      eval('xx');
      assert(false);  // should not reach here
    } catch (e2) {
      assert(e2 instanceof ReferenceError);
    }
  })();
}
assert(caught);

