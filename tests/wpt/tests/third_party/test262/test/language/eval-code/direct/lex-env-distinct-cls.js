// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-performeval
description: >
    Direct eval code creates a new declarative environment for lexically-scoped
    declarations (class)
info: |
    [...]
    9. If direct is true, then
       a. Let lexEnv be NewDeclarativeEnvironment(ctx's LexicalEnvironment).
    [...]
features: [class]
---*/

class outside {}

eval('class outside {}');
eval('"use strict"; class outside {}');

eval('class xNonStrict {}');

assert.sameValue(typeof xNonStrict, 'undefined');
assert.throws(ReferenceError, function() {
  xNonStrict;
});

eval('"use strict"; class xStrict {}');

assert.sameValue(typeof xStrict, 'undefined');
assert.throws(ReferenceError, function() {
  xStrict;
});
