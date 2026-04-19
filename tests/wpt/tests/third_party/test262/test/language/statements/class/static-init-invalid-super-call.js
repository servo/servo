// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions-static-semantics-early-errors
description: Block cannot contain SuperCall
info: |
  ClassStaticBlock : static { ClassStaticBlockBody }

  - It is a Syntax Error if HasDirectSuper of ClassStaticBlock is true.
negative:
  phase: parse
  type: SyntaxError
features: [class-static-block]
---*/

$DONOTEVALUATE();

class C {
  static {
    super();
  }
}
