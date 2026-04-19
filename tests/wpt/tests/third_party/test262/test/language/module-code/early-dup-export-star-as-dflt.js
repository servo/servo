// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-semantics-static-semantics-early-errors
description: >
    It is a Syntax Error if the ExportedNames of ModuleItemList contains any
    duplicate entries.
flags: [module]
features: [export-star-as-namespace-from-module]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var x;
export default x;
export * as default from './early-dup-export-start-as-dflt.js';
