// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-performeval
description: >
    Indirect eval code sets the new declarative environment's outer environment
    to the global environment.
info: |
    [...]
    9. If direct is true, then
       [...]
    10. Else,
        a. Let lexEnv be NewDeclarativeEnvironment(evalRealm.[[GlobalEnv]]).
    [...]
features: [let]
---*/

var actualStrict;
var actualNonStrict;

let x = 'outside';
{
  let x = 'inside';
  actualNonStrict = (0,eval)('x;');
  actualStrict = (0,eval)('"use strict"; x;');
}

assert.sameValue(actualNonStrict, 'outside', 'non strict mode');
assert.sameValue(actualStrict, 'outside', 'strict mode');
