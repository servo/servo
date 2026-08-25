// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-scripts-static-semantics-early-errors
es6id: 15.1.1
description: Global code may not contain SuperProperty
info: |
  - It is a Syntax Error if StatementList Contains super unless the source code
    containing super is eval code that is being processed by a direct eval that
    is contained in function code that is not the function code of an
    ArrowFunction.
negative:
  phase: parse
  type: SyntaxError
features: [super]
---*/

$DONOTEVALUATE();

super.property;
