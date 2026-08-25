// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions-static-semantics-early-errors
description: Block cannot declare duplicate labels
info: |
  ClassStaticBlockBody : ClassStaticBlockStatementList

  - It is a Syntax Error if ContainsDuplicateLabels of
    ClassStaticBlockStatementList with argument « » is true.
negative:
  phase: parse
  type: SyntaxError
features: [class-static-block]
---*/

$DONOTEVALUATE();

class C {
  static {
    x: x: 0;
  }
}

