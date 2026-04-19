// Copyright (C) 2021 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-wrapped-function-exotic-objects-call-thisargument-argumentslist
description: >
  WrappedFunction throws a TypeError from its creation realm.
features: [ShadowRealm, cross-realm, Reflect]
---*/

assert.sameValue(
  typeof ShadowRealm.prototype.evaluate,
  'function',
  'This test must fail if ShadowRealm.prototype.evaluate is not a function'
);

var other = $262.createRealm().global;
var OtherTypeError = other.TypeError;
var OtherShadowRealm = other.ShadowRealm;

var yetAnother = $262.createRealm().global;
var YetAnotherTypeError = yetAnother.TypeError;
var YetAnotherShadowRealm = yetAnother.ShadowRealm;

var realm = Reflect.construct(OtherShadowRealm, []);

{
  let wrappedFunction = realm.evaluate('() => {}');
  let wrappedFunction2 = realm.evaluate('() => globalThis');

  assert.throws(OtherTypeError, () => wrappedFunction(1, globalThis), 'throws TypeError if arguments are not wrappable');
  assert.throws(OtherTypeError, () => wrappedFunction2(), 'throws TypeError if return value is not wrappable');
}

{
  let wrappedFunction = YetAnotherShadowRealm.prototype.evaluate.call(realm, '() => {}');
  let wrappedFunction2 = YetAnotherShadowRealm.prototype.evaluate.call(realm, '() => globalThis');
  assert.throws(YetAnotherTypeError, () => wrappedFunction(1, globalThis), 'throws TypeError if arguments are not wrappable');
  assert.throws(YetAnotherTypeError, () => wrappedFunction2(), 'throws TypeError if return value is not wrappable');
}
