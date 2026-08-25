// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The default property is not set the if the module doesn't export any default
esid: sec-finishdynamicimport
features: [dynamic-import]
flags: [async]
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
---*/

import('./empty_FIXTURE.js').then(ns => {

    assert.sameValue(Object.prototype.hasOwnProperty.call(ns, 'default'), false);

}).then($DONE, $DONE).catch($DONE);
