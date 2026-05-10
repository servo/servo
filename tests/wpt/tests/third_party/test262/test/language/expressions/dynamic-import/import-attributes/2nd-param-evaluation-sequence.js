// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Evaluates parameters in correct sequence
esid: sec-import-call-runtime-semantics-evaluation
info: |
  2.1.1.1 EvaluateImportCall ( specifierExpression [ , optionsExpression ] )
    1. Let referencingScriptOrModule be ! GetActiveScriptOrModule().
    2. Let specifierRef be the result of evaluating specifierExpression.
    3. Let specifier be ? GetValue(specifierRef).
    4. If optionsExpression is present, then
       a. Let optionsRef be the result of evaluating optionsExpression.
       b. Let options be ? GetValue(optionsRef).
    [...]
features: [dynamic-import, import-attributes]
---*/

var log = [];

import(log.push('first'), (log.push('second'), undefined))
  .then(null, function() {});

assert.sameValue(log.length, 2);
assert.sameValue(log[0], 'first');
assert.sameValue(log[1], 'second');
