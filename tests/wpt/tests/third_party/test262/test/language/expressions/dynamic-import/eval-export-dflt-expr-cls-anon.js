// Copyright (C) 2018 Rick Waldron. All rights reserved.
// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Default AssignmentExpression (which can be recognized as an "anonymous"
    class declaration) is correctly initialized upon evaluation
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
features: [dynamic-import]
---*/

export default (class { valueOf() { return 45; } });
import('./eval-export-dflt-expr-cls-anon.js').then(imported => {
  assert.sameValue(new imported.default().valueOf(), 45, 'binding initialized');
  assert.sameValue(imported.default.name, 'default', 'correct name is assigned');
}).then($DONE, $DONE).catch($DONE);
