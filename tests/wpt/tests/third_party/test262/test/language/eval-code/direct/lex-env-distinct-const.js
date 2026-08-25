// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-performeval
description: >
    Direct eval code creates a new declarative environment for lexically-scoped
    declarations (const)
info: |
    [...]
    9. If direct is true, then
       a. Let lexEnv be NewDeclarativeEnvironment(ctx's LexicalEnvironment).
    [...]
features: [const]
---*/

const outside = null;

eval('const outside = null;');
eval('"use strict"; const outside = null;');

eval('const xNonStrict = null;');

assert.sameValue(typeof xNonStrict, 'undefined');
assert.throws(ReferenceError, function() {
  xNonStrict;
});

eval('"use strict"; const xStrict = null;');

assert.sameValue(typeof xStrict, 'undefined');
assert.throws(ReferenceError, function() {
  xStrict;
});
