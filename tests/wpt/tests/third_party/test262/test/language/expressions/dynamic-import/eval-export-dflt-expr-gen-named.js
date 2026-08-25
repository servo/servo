// Copyright (C) 2018 Rick Waldron. All rights reserved.
// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Default AssignmentExpression (which can be recognized as a "named"
    generator function declaration) is correctly initialized upon evaluation
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
flags: [async, module]
features: [dynamic-import, generators]
---*/

export default (function* gName() { return 42; });
import('./eval-export-dflt-expr-gen-named.js').then(imported => {
  assert.sameValue(imported.default().next().value, 42, 'binding initialized');
  assert.sameValue(imported.default.name, 'gName', 'correct name is assigned');
}).then($DONE, $DONE).catch($DONE);
