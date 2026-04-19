// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Resolve multiple imports through a for await loop in an async generator
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
features: [dynamic-import, async-iteration]
includes: [asyncHelpers.js]
---*/

async function * agen() {
  for await (let imported of [
      import('./for-await-resolution-and-error-a_FIXTURE.js'),
      import('./for-await-resolution-and-error-b_FIXTURE.js'),
      import('./for-await-resolution-and-error-poisoned_FIXTURE.js'),
    ]) {
    yield imported.x;
  }
}

var aiter = agen();

async function fn() {
    var a = aiter.next();
    var b = aiter.next();
    var c = aiter.next();

    assert.sameValue((await a).value, 42);
    assert.sameValue((await b).value, 39);

    var error;
    try {
        await c;
    } catch (e) {
        error = e;
    }

    assert.sameValue(error, 'foo');
}

asyncTest(fn);
