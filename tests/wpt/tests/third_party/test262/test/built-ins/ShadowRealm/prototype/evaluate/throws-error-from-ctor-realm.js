// Copyright (C) 2021 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-shadowrealm.prototype.evaluate
description: >
  ShadowRealm.prototype.evaluate throws a TypeError from ShadowRealm's creation realm.
features: [ShadowRealm, cross-realm, Reflect]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

var other = $262.createRealm().global;
var OtherTypeError = other.TypeError;
var OtherSyntaxError = other.SyntaxError;
var OtherShadowRealm = other.ShadowRealm;

var realm = Reflect.construct(OtherShadowRealm, []);

assert.throws(OtherTypeError, () => realm.evaluate('globalThis'), 'throws a TypeError if return value can not be wrapped');
assert.throws(OtherTypeError, () => realm.evaluate('throw new Error()'), 'throws a TypeError if completion is abrupt');

assert.throws(OtherTypeError, () => realm.evaluate(1), 'throws a TypeError if sourceText is not a string');
assert.throws(OtherSyntaxError, () => realm.evaluate('...'), 'throws a SyntaxError if the sourceText is not valid');

const bogus = {};
assert.throws(OtherTypeError, function() {
  realm.evaluate.call(bogus, 'This is invalid code and should not be evaluated');
}, 'throws a TypeError if this is not a ShadowRealm object');
