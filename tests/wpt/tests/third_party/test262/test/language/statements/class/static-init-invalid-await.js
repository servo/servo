// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions
description: The "Await" parsing context does not apply to the block's statement list
info: |
  Syntax

  [...]

  ClassStaticBlockStatementList :
     StatementList[~Yield, +Await, ~Return]opt

  ## 15.7.1 Static Semantics: Early Errors

  ClassStaticBlockBody : ClassStaticBlockStatementList

  - It is a Syntax Error if ContainsAwait of ClassStaticBlockStatementList is true.
negative:
  phase: parse
  type: SyntaxError
features: [class-static-block]
---*/

$DONOTEVALUATE();

async function f() {
  class C {
    static {
      await 0;
    }
  }
}
