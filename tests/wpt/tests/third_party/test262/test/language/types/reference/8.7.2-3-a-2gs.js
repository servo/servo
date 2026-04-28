// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 8.7.2-3-a-2gs
description: >
    Strict Mode - 'runtime' error is thrown before LeftHandSide
    evaluates to an unresolvable Reference
flags: [onlyStrict]
---*/

assert.throws(Test262Error, function() {
  throw new Test262Error();
  b = 11;
});
