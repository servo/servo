// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-performeval
description: >
    Indirect eval code creates a new declarative environment for
    lexically-scoped declarations (const)
info: |
    [...]
    9. If direct is true, then
       [...]
    10. Else,
        a. Let lexEnv be NewDeclarativeEnvironment(evalRealm.[[GlobalEnv]]).
    [...]
features: [const]
---*/

const outside = null;

(0,eval)('const outside = null;');
(0,eval)('"use strict"; const outside = null;');

(0,eval)('const xNonStrict = null;');

assert.sameValue(typeof xNonStrict, 'undefined');
assert.throws(ReferenceError, function() {
  xNonStrict;
});

(0,eval)('"use strict"; const xStrict = null;');

assert.sameValue(typeof xStrict, 'undefined');
assert.throws(ReferenceError, function() {
  xStrict;
});
