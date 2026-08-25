// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Resolve multiple imports through a for await loop in an async generator yielding imports
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

async function * agen1() {
    yield import('./for-await-resolution-and-error-a_FIXTURE.js');
    yield import('./for-await-resolution-and-error-b_FIXTURE.js');
    yield import('./for-await-resolution-and-error-poisoned_FIXTURE.js');
}

async function * agen2() {
    yield await import('./for-await-resolution-and-error-a_FIXTURE.js');
    yield await import('./for-await-resolution-and-error-b_FIXTURE.js');
    yield await import('./for-await-resolution-and-error-poisoned_FIXTURE.js');
}

var aiter1 = agen1();
var aiter2 = agen2();

async function fn() {
    var a = aiter1.next();
    var b = aiter1.next();
    var c = aiter1.next();
    var d = aiter2.next();
    var e = aiter2.next();
    var f = aiter2.next();

    assert.sameValue((await a).value.x, 42, 'a');
    assert.sameValue((await b).value.x, 39, 'b');

    var error;
    try {
        await c;
    } catch (err) {
        error = err;
    }

    assert.sameValue(error, 'foo', 'c');

    assert.sameValue((await d).value.x, 42, 'd');
    assert.sameValue((await e).value.x, 39, 'e');

    error = null;
    try {
        await f;
    } catch (err) {
        error = err;
    }

    assert.sameValue(error, 'foo', 'f');
}

asyncTest(fn);
