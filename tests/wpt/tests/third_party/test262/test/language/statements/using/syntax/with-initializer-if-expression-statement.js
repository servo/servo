// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-if-statement
description: >
    using declarations with initialisers in statement positions:
    if ( Expression ) Statement
negative:
  phase: parse
  type: SyntaxError
features: [explicit-resource-management]
---*/

$DONOTEVALUATE();
if (true) using x = null;
