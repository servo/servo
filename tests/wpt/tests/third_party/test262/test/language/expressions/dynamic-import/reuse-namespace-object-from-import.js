// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Reuse the resolved namespace object already imported from a static import
esid: sec-import-call-runtime-semantics-evaluation
info: |
    Runtime Semantics: FinishDynamicImport ( referencingScriptOrModule, specifier, promiseCapability, completion )

        1. If completion is an abrupt completion, ...
        2. Otherwise,
            ...
            d. Let namespace be GetModuleNamespace(moduleRecord).
            e. If namespace is an abrupt completion, perform ! Call(promiseCapability.[[Reject]], undefined, « namespace.[[Value]] »).
            f. Otherwise, perform ! Call(promiseCapability.[[Resolve]], undefined, « namespace.[[Value]] »).

    Runtime Semantics: GetModuleNamespace ( module )

        ...
        3. Let namespace be module.[[Namespace]].
        4. If namespace is undefined, then
            ...
            d. Set namespace to ModuleNamespaceCreate(module, unambiguousNames).
        5. Return namespace.
features: [dynamic-import]
flags: [async, module]
---*/

import * as ns from './module-code_FIXTURE.js';

Promise.all([
    import('./module-code_FIXTURE.js'),
    import('./module-code_FIXTURE.js'),
]).then(([a, b]) => {
    assert.sameValue(a, b, 'it returns the same namespace are the same');
    assert.sameValue(a, ns, 'dynamic imported a is the same object as ns');
    assert.sameValue(b, ns, 'dynamic imported b is the same object as ns');
}).then($DONE, $DONE);
