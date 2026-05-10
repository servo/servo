// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-semantics-static-semantics-early-errors
description: >
    It is a Syntax Error if the LexicallyDeclaredNames of ModuleItemList
    contains any duplicate entries.
flags: [module]
features: [let, const]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

let x;
const x = 0;
