// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.1-6
description: >
    12.1 - block '{ StatementListopt };' is not allowed:
    if-else-if-else
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

if{};else if{};else{}
