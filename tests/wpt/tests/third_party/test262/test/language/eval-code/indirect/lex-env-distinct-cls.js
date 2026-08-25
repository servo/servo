// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-performeval
description: >
    Indirect eval code creates a new declarative environment for
    lexically-scoped declarations (class)
info: |
    [...]
    9. If direct is true, then
       [...]
    10. Else,
        a. Let lexEnv be NewDeclarativeEnvironment(evalRealm.[[GlobalEnv]]).
    [...]
features: [class]
---*/

class outside {}

(0,eval)('class outside {}');
(0,eval)('"use strict"; class outside {}');

(0,eval)('class xNonStrict {}');

assert.sameValue(typeof xNonStrict, 'undefined');
assert.throws(ReferenceError, function() {
  xNonStrict;
});

(0,eval)('"use strict"; class xStrict {}');

assert.sameValue(typeof xStrict, 'undefined');
assert.throws(ReferenceError, function() {
  xStrict;
});
