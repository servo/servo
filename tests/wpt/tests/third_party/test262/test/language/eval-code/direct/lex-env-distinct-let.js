// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-performeval
description: >
    Direct eval code creates a new declarative environment for lexically-scoped
    declarations (let)
info: |
    [...]
    9. If direct is true, then
       a. Let lexEnv be NewDeclarativeEnvironment(ctx's LexicalEnvironment).
    [...]
features: [let]
---*/

let outside = 23;

eval('let outside;');
eval('"use strict"; let outside;');

eval('let xNonStrict = 3;');

assert.sameValue(typeof xNonStrict, 'undefined');
assert.throws(ReferenceError, function() {
  xNonStrict;
});

eval('"use strict"; let xStrict = 3;');

assert.sameValue(typeof xStrict, 'undefined');
assert.throws(ReferenceError, function() {
  xStrict;
});
