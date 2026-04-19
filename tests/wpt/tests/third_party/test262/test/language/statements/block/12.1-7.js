// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.1-7
description: "12.1 - block '{ StatementListopt };' is not allowed: do-while"
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

do{};while()
