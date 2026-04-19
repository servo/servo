// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Forwards "throw" completion when evaluating second parameter
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

var beforeCount = 0;
var afterCount = 0;
function throwError() {
  throw new Test262Error();
}

assert.throws(Test262Error, function() {
  beforeCount += 1, import('', throwError()), afterCount += 1;
});

assert.sameValue(beforeCount, 1);
assert.sameValue(afterCount, 0);
