// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-class-definitions-static-semantics-early-errors
description: The `await` keyword is interpreted as an identifier within arrow function bodies
info: |
  ClassStaticBlockBody : ClassStaticBlockStatementList

  [...]
  - It is a Syntax Error if ContainsAwait of ClassStaticBlockStatementList is true.
features: [class-static-block, explicit-resource-management]
---*/

class C {
  static {
    (() => { using await = null; });
  }
}
