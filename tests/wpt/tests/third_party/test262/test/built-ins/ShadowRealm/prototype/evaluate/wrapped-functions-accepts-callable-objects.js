// Copyright (C) 2021 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate accepts callable objects
features: [ShadowRealm]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

const r = new ShadowRealm();

assert.sameValue(typeof r.evaluate('function fn() {} fn'), 'function', 'value from a fn declaration');
assert.sameValue(typeof r.evaluate('(function() {})'), 'function', 'function expression');
assert.sameValue(typeof r.evaluate('(async function() {})'), 'function', 'async function expression');
assert.sameValue(typeof r.evaluate('(function*() {})'), 'function', 'generator expression');
assert.sameValue(typeof r.evaluate('(async function*() {})'), 'function', 'async generator expression');
assert.sameValue(typeof r.evaluate('() => {}'), 'function', 'arrow function');
assert.sameValue(typeof r.evaluate('new Proxy(() => {}, { apply() {} })'), 'function', 'callable proxy');
