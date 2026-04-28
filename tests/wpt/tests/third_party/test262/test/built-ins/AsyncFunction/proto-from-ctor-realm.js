// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-async-function-constructor-arguments
description: Default [[Prototype]] value derived from realm of the NewTarget.
info: |
  AsyncFunction ( p1, p2, … , pn, body )

  ...
  3. Return CreateDynamicFunction(C, NewTarget, "async", args).

  Runtime Semantics: CreateDynamicFunction ( constructor, newTarget, kind, args )

  ...
  9. Else if kind is "async", then
    ...
    c. Let fallbackProto be "%AsyncFunction.prototype%".
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
features: [async-functions, cross-realm, Reflect, Symbol]
---*/

var AsyncFunction = Object.getPrototypeOf(async function() {}).constructor;
var other = $262.createRealm().global;
var OtherAsyncFunction = Object.getPrototypeOf(other.eval('(0, async function() {})')).constructor;
var newTarget = new other.Function();
var fn;

newTarget.prototype = undefined;
fn = Reflect.construct(AsyncFunction, [], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), OtherAsyncFunction.prototype, 'newTarget.prototype is undefined');

newTarget.prototype = null;
fn = Reflect.construct(AsyncFunction, [], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), OtherAsyncFunction.prototype, 'newTarget.prototype is null');

newTarget.prototype = true;
fn = Reflect.construct(AsyncFunction, [], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), OtherAsyncFunction.prototype, 'newTarget.prototype is a Boolean');

newTarget.prototype = '';
fn = Reflect.construct(AsyncFunction, [], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), OtherAsyncFunction.prototype, 'newTarget.prototype is a String');

newTarget.prototype = Symbol();
fn = Reflect.construct(AsyncFunction, [], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), OtherAsyncFunction.prototype, 'newTarget.prototype is a Symbol');

newTarget.prototype = 1;
fn = Reflect.construct(AsyncFunction, [], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), OtherAsyncFunction.prototype, 'newTarget.prototype is a Number');
