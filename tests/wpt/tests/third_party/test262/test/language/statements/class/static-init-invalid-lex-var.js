// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions-static-semantics-early-errors
description: Block cannot declare a lexically-scoped binding and function-scoped binding with the same name.
info: |
  ClassStaticBlockBody : ClassStaticBlockStatementList

  - It is a Syntax Error if any element of the LexicallyDeclaredNames of
    ClassStaticBlockStatementList also occurs in the VarDeclaredNames of
    ClassStaticBlockStatementList.
negative:
  phase: parse
  type: SyntaxError
features: [class-static-block]
---*/

$DONOTEVALUATE();

class C {
  static {
    let x;
    var x;
  }
}
