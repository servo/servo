// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate wrapped function from return values share no identity.
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

r.evaluate(`
function fn() { return 42; }
globalThis.arrow = x => x * 2;
globalThis.pFn = new Proxy(fn, {
    apply() {
        pFn.used = 1;
        return 39;
    }
});
async function aFn() {
    return 1;
}

function * genFn() {
    return 1;
}

fn.x = 'secrets';
arrow.x = 'secrets';
pFn.x = 'secrets';
aFn.x = 'secrets';
genFn.x = 'secrets';
`)

const wrappedOrdinary = r.evaluate('() => fn')();
assert.sameValue(typeof wrappedOrdinary, 'function', 'ordinary function wrapped');
assert.sameValue(wrappedOrdinary(), 42, 'ordinary, return');
assert.sameValue(wrappedOrdinary.x, undefined, 'ordinary, no property shared');
assert.sameValue(Object.prototype.hasOwnProperty.call(wrappedOrdinary, 'x'), false, 'ordinary, no own property shared');

const wrappedArrow = r.evaluate('() => arrow')();
assert.sameValue(typeof wrappedArrow, 'function', 'arrow function wrapped');
assert.sameValue(wrappedArrow(7), 14, 'arrow function, return');
assert.sameValue(wrappedArrow.x, undefined, 'arrow function, no property');
assert.sameValue(Object.prototype.hasOwnProperty.call(wrappedArrow, 'x'), false, 'arrow function, no own property shared');

const wrappedProxied = r.evaluate('() => pFn')();
assert.sameValue(typeof wrappedProxied, 'function', 'proxied ordinary function wrapped');
assert.sameValue(r.evaluate('pFn.used'), undefined, 'pFn not called yet');
assert.sameValue(wrappedProxied(), 39, 'return of the proxied callable');
assert.sameValue(r.evaluate('pFn.used'), 1, 'pfn called');
assert.sameValue(wrappedProxied.x, undefined, 'proxy callable, no property');
assert.sameValue(Object.prototype.hasOwnProperty.call(wrappedProxied, 'x'), false, 'proxy callable, no own property shared');

const wrappedAsync = r.evaluate('() => aFn')();
assert.sameValue(typeof wrappedAsync, 'function', 'async function wrapped');
assert.throws(TypeError, () => wrappedAsync(), 'wrapped function cannot return non callable object');
assert.sameValue(wrappedAsync.x, undefined, 'async function, no property');
assert.sameValue(Object.prototype.hasOwnProperty.call(wrappedAsync, 'x'), false, 'async function, no own property shared');

const wrappedGenerator = r.evaluate('() => genFn')();
assert.sameValue(typeof wrappedGenerator, 'function', 'gen function wrapped');
assert.throws(TypeError, () => wrappedGenerator(), 'wrapped function cannot return non callable object');
assert.sameValue(wrappedGenerator.x, undefined, 'generator, no property');
assert.sameValue(Object.prototype.hasOwnProperty.call(wrappedGenerator, 'x'), false, 'generator, no own property shared');
