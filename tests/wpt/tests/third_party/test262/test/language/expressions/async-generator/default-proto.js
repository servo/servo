// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asyncgenerator-definitions-evaluatebody
description: >
  Default [[Prototype]] value derived from realm of the async generator function.
info: |
  Runtime Semantics: EvaluateBody

  ...
  2. Let generator be ? OrdinaryCreateFromConstructor(functionObject, "%AsyncGeneratorPrototype%", « ... »).
  3. Perform ! AsyncGeneratorStart(generator, FunctionBody).
  4. Return Completion { [[Type]]: return, [[Value]]: generator, [[Target]]: empty }.

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

  GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  ...
  3. Let proto be ? Get(constructor, 'prototype').
  4. If Type(proto) is not Object, then
    a. Let realm be ? GetFunctionRealm(constructor).
    b. Set proto to realm's intrinsic object named intrinsicDefaultProto.
  5. Return proto.
features: [async-iteration, Symbol]
---*/

var fn = async function* () {};
var AsyncGeneratorPrototype = Object.getPrototypeOf(fn.prototype);

fn.prototype = undefined;
assert.sameValue(Object.getPrototypeOf(fn()), AsyncGeneratorPrototype, 'fn.prototype is undefined');

fn.prototype = null;
assert.sameValue(Object.getPrototypeOf(fn()), AsyncGeneratorPrototype, 'fn.prototype is null');

fn.prototype = false;
assert.sameValue(Object.getPrototypeOf(fn()), AsyncGeneratorPrototype, 'fn.prototype is a Boolean');

fn.prototype = '';
assert.sameValue(Object.getPrototypeOf(fn()), AsyncGeneratorPrototype, 'fn.prototype is a String');

fn.prototype = Symbol();
assert.sameValue(Object.getPrototypeOf(fn()), AsyncGeneratorPrototype, 'fn.prototype is a Symbol');

fn.prototype = 1;
assert.sameValue(Object.getPrototypeOf(fn()), AsyncGeneratorPrototype, 'fn.prototype is a Number');
