// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Resolve imports after a binding update
esid: sec-finishdynamicimport
info: |
    Runtime Semantics: FinishDynamicImport ( referencingScriptOrModule, specifier, promiseCapability, completion )

    2. Otherwise,
        a. Assert: completion is a normal completion and completion.[[Value]] is undefined.
        b. Let moduleRecord be ! HostResolveImportedModule(referencingScriptOrModule, specifier).
        c. Assert: Evaluate has already been invoked on moduleRecord and successfully completed.
        d. Let namespace be GetModuleNamespace(moduleRecord).
        ...
        f. Otherwise, perform ! Call(promiseCapability.[[Resolve]], undefined, « namespace.[[Value]] »).
flags: [async]
features: [dynamic-import]
includes: [asyncHelpers.js]
---*/

async function fn() {
    const first = await import('./update-to-dynamic-import_FIXTURE.js');
    assert.sameValue(first.x, 'first', 'the other module has not been evaluated yet');

    const other = await first.default();

    assert.sameValue(first.x, 'other', 'the other module is only evaluated after calling import()');
    assert.sameValue(other.default, 42);
}

asyncTest(fn);
