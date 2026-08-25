// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions
description: The block's statement list is optional
info: |
  Syntax

  [...]

  ClassStaticBlockStatementList :
     StatementList[~Yield, +Await, ~Return]opt
features: [class-static-block]
---*/

class C {
  static {}
}
