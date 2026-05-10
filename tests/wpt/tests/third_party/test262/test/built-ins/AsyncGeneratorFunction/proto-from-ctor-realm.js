// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgeneratorfunction
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  AsyncGeneratorFunction ( p1, p2, … , pn, body )

  ...
  3. Return ? CreateDynamicFunction(C, NewTarget, "async generator", args).

  Runtime Semantics: CreateDynamicFunction ( constructor, newTarget, kind, args )

  ...
  10. Else,
    a. Assert: kind is "async generator".
    ...
    d. Let fallbackProto be "%AsyncGenerator%".
  ...
  18. Let proto be ? GetPrototypeFromConstructor(newTarget, fallbackProto).
  ...

  GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  ...
  3. Let proto be ? Get(constructor, "prototype").
  4. If Type(proto) is not Object, then
    a. Let realm be ? GetFunctionRealm(constructor).
    b. Set proto to realm's intrinsic object named intrinsicDefaultProto.
  5. Return proto.
features: [async-iteration, cross-realm, Reflect, Symbol]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;
var other = $262.createRealm().global;
var OtherAsyncGeneratorFunction = Object.getPrototypeOf(other.eval('(0, async function* () {})')).constructor;
var newTarget = new other.Function();
var fn;

newTarget.prototype = undefined;
fn = Reflect.construct(AsyncGeneratorFunction, [], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), OtherAsyncGeneratorFunction.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
fn = Reflect.construct(AsyncGeneratorFunction, [], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), OtherAsyncGeneratorFunction.prototype, 'newTarget.prototype is null');

newTarget.prototype = true;
fn = Reflect.construct(AsyncGeneratorFunction, [], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), OtherAsyncGeneratorFunction.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = '';
fn = Reflect.construct(AsyncGeneratorFunction, [], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), OtherAsyncGeneratorFunction.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
fn = Reflect.construct(AsyncGeneratorFunction, [], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), OtherAsyncGeneratorFunction.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = 1;
fn = Reflect.construct(AsyncGeneratorFunction, [], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), OtherAsyncGeneratorFunction.prototype, 'newTarget.prototype is a Number');
