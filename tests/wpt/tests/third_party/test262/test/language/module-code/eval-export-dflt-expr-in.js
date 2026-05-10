// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    The `in` operator may occur within an exported AssignmentExpression
esid: sec-moduleevaluation
info: |
    [...]
    16. Let result be the result of evaluating module.[[ECMAScriptCode]].
    [...]

    15.2.3 Exports

    Syntax

    ExportDeclaration :

    export default [lookahead ∉ { function, class }] AssignmentExpression[In];
flags: [module]
---*/

var x = { x: true };

export default 'x' in x;
import f from './eval-export-dflt-expr-in.js';

assert.sameValue(f, true);
