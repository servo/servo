// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Dynamic Import receives an AssignmentExpression (ImportMeta)
esid: prod-ImportCall
info: |
    ImportCall [Yield]:
        import ( AssignmentExpression[+In, ?Yield] )

    Runtime Semantics: Evaluation

    ImportCall : import ( AssignmentExpression )

    ...
    5. Let specifierString be ToString(specifier).
    6. IfAbruptRejectPromise(specifierString, promiseCapability).
features: [dynamic-import, import.meta]
flags: [module, async]
---*/

const p = import(import.meta);

// We can at least assert p is a promise.
assert.sameValue(Promise.resolve(p), p, 'Assert that p is a promise');

// The keys of import.meta are implementation defined, but we know its
// [[Prototype]] is null. In this case, import() should reject the
// promise it returns, unless a toPrimitive related method is set.
if (!Object.prototype.hasOwnProperty.call(import.meta, 'toString') &&
        !Object.prototype.hasOwnProperty.call(import.meta, 'valueOf') &&
        !Object.prototype.hasOwnProperty.call(import.meta, Symbol.toPrimitive)) {
    p.catch(error => assert.sameValue(error.constructor, TypeError, 'import() cannot resolve import.meta')).then($DONE, $DONE);
} else {
    $DONE();
}
