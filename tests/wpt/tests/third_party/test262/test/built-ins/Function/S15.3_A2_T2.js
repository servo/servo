// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Since applying the "call" method to Function constructor themself leads
    to creating a new function instance, the second argument must be a valid
    function body
es5id: 15.3_A2_T2
description: Checking if executing "Function.call(this, "var #x  = 1;")" fails
---*/

try {
  Function.call(this, "var #x  = 1;");
} catch (e) {
  assert(
    e instanceof SyntaxError,
    'The result of evaluating (e instanceof SyntaxError) is expected to be true'
  );
}
