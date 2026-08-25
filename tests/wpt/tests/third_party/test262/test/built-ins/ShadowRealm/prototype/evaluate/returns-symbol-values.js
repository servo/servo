// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate returns symbol values
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();
const s = r.evaluate('Symbol("foobar")');

assert.sameValue(typeof s, 'symbol');
assert.sameValue(s.constructor, Symbol, 'primitive does not expose other ShadowRealm constructor');
assert.sameValue(Object.getPrototypeOf(s), Symbol.prototype);
assert.sameValue(Symbol.prototype.toString.call(s), 'Symbol(foobar)');

const shadowX = r.evaluate('Symbol.for("my symbol name")');
const myX = Symbol.for('my symbol name')

assert.sameValue(
  shadowX,
  myX,
  'The shadow realms observes the symbol global registry used in Symbol.for'
);

assert.sameValue(
  Symbol.keyFor(shadowX),
  'my symbol name',
  'Symbol.keyFor observes the string key name of a symbol originally registered in the shadow realm'
);

assert.sameValue(
  Symbol.keyFor(s),
  undefined,
  'Symbol.keyFor cannot find a key for a regular symbol created in the shadow realm'
);

const { get: description } = Object.getOwnPropertyDescriptor(Symbol.prototype, 'description');

assert.sameValue(
  description.call(s),
  'foobar',
  'get description for the symbol created in the shadow realm'
);
