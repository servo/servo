// Copyright (c) 2018 Mike Pennisi.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: use-strict-directive
es5id: 10.1.1-2-s
description: >
    Strict Mode - Use Strict Directive Prologue is ''use strict''
    which lost the last character ';'
negative:
  phase: parse
  type: SyntaxError
flags: [noStrict]
---*/

$DONOTEVALUATE();

function fun() {
  "use strict"
  var static;
}
