// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Default AssignmentExpression (which can be recognized as a "named" function
    declaration) is correctly initialized upon evaluation
esid: sec-moduleevaluation
info: |
    [...]
    16. Let result be the result of evaluating module.[[ECMAScriptCode]].
    [...]

    15.2.3.11 Runtime Semantics: Evaluation

    ExportDeclaration : export default AssignmentExpression;

    [...]
    3. If IsAnonymousFunctionDefinition(AssignmentExpression) is true, then
       a. Let hasNameProperty be ? HasOwnProperty(value, "name").
       b. If hasNameProperty is false, perform SetFunctionName(value,
          "default").
    4. Let env be the running execution context's LexicalEnvironment.
    5. Perform ? InitializeBoundName("*default*", value, env).
    [...]
flags: [module]
---*/

export default (function fName() { return 7; });
import f from './eval-export-dflt-expr-fn-named.js';

assert.sameValue(f(), 7, 'binding initialized');
assert.sameValue(f.name, 'fName', 'correct name is assigned');
