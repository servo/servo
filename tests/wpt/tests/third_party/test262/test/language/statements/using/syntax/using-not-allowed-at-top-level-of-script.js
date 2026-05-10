// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-let-const-using-and-await-using-declarations-static-semantics-early-errors
description: >
    using declarations not allowed at the top level of a Script
info: |
  UsingDeclaration : using BindingList ;

  - It is a Syntax Error if the goal symbol is Script and UsingDeclaration is not contained, either directly or 
    indirectly, within a Block, ForStatement, ForInOfStatement, FunctionBody, GeneratorBody, 
    AsyncGeneratorBody, AsyncFunctionBody, ClassStaticBlockBody, or ClassBody.

negative:
  phase: parse
  type: SyntaxError
features: [explicit-resource-management]
---*/

$DONOTEVALUATE();
using x = null;
