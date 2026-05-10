// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Return Abrupt from the GetValue evaluation on the given AssignmentExpression
esid: sec-import-call-runtime-semantics-evaluation
info: |
    Import Calls

    Runtime Semantics: Evaluation
    
    ImportCall : import(AssignmentExpression)
    
    1. Let referencingScriptOrModule be ! GetActiveScriptOrModule().
    2. Let argRef be the result of evaluating AssignmentExpression.
    3. Let specifier be ? GetValue(argRef).
    4. Let promiseCapability be ! NewPromiseCapability(%Promise%).
    5. Let specifierString be ToString(specifier).
    6. IfAbruptRejectPromise(specifierString, promiseCapability).
    7. Perform ! HostImportModuleDynamically(referencingScriptOrModule, specifierString, promiseCapability).
    8. Return promiseCapability.[[Promise]].
features: [dynamic-import]
---*/

const obj = {
    get err() {
        throw new Test262Error('catpure this on evaluation')
    }
}

assert.throws(Test262Error, function() {
    import(obj.err);
}, 'Custom Error getting property value');

assert.throws(ReferenceError, function() {
    import(refErr);
}, 'bad reference');
