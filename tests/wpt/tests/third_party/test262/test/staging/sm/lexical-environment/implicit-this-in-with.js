// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Test that callees that resolve to bindings on the global object or the
// global lexical environment get an 'undefined' this inside with scopes.

let g = function () { "use strict"; assert.sameValue(this, undefined); }
function f() { "use strict"; assert.sameValue(this, undefined); }

with ({}) { 
  // f is resolved on the global object
  f();
  // g is resolved on the global lexical environment
  g();
}

f();
g();

