// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Import a custom toString and valueOf bindings
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
    const str = await import('./custom-tostring_FIXTURE.js');
    const value = await import('./custom-valueof_FIXTURE.js');

    assert.sameValue(String(str), '1612', 'namespace uses the imported toString');
    assert.sameValue(Number(str), 1612, 'namespace fallsback to toString as its prototype is null');

    assert.sameValue(Number(value), 42, 'namespace uses the imported valueOf');
    assert.sameValue(String(value), '42', 'namespace fallsback to valueOf as its prototype is null');
}

asyncTest(fn);
