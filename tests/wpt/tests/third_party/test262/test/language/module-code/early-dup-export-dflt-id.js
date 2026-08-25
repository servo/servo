// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-semantics-static-semantics-early-errors
es6id: 15.2.1.1
description: >
    It is a Syntax Error if the ExportedNames of ModuleItemList contains any
    duplicate entries.
flags: [module]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var x, y;
export default x;
export { y as default };
