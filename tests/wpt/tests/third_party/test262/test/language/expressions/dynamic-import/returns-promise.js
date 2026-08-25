// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    ImportCall returns a promise
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
includes: [fnGlobalObject.js]
---*/

const originalPromise = Promise;

fnGlobalObject().Promise = function() {
    throw "This should not be called";
};

const p = import('./dynamic-import-module_FIXTURE.js');

assert.sameValue(p.constructor, originalPromise, 'constructor is %Promise%');
assert.sameValue(Object.getPrototypeOf(p), originalPromise.prototype, 'prototype is %PromisePrototype%');
assert.sameValue(p.then, originalPromise.prototype.then, 'preserves the original `then` method');
assert.sameValue(p.catch, originalPromise.prototype.catch, 'preserves the original `catch` method');
assert.sameValue(p.finally, originalPromise.prototype.finally, 'preserves the original `finally` method');

assert.sameValue(
    Object.prototype.hasOwnProperty.call(p, 'then'), false,
    'returned promise has no own property then'
);

assert.sameValue(
    Object.prototype.hasOwnProperty.call(p, 'catch'), false,
    'returned promise has no own property catch'
);

assert.sameValue(
    Object.prototype.hasOwnProperty.call(p, 'finally'), false,
    'returned promise has no own property finally'
);
