// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Appearing of "continue" within a function call that is within an
    IterationStatement yields SyntaxError
es5id: 12.7_A6
description: Using labaled "continue Identifier" within a function body
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var x=0,y=0;

LABEL1 : do {
    x++;
    (function(){continue LABEL1;})();
    y++;
} while(0);
