// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions-static-semantics-early-errors
description: The `await` keyword is disallowed as a BindingIdentifier
info: |
  ClassStaticBlockBody : ClassStaticBlockStatementList

  [...]
  - It is a Syntax Error if ContainsAwait of ClassStaticBlockStatementList is true.
features: [class-static-block]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

class C {
  static {
    (class await {});
  }
}
