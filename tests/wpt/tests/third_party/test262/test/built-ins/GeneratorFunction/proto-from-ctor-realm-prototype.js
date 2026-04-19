// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdynamicfunction
description: >
  While default [[Prototype]] value derives from realm of the newTarget,
  "prototype" object inherits from %Object.prototype% of constructor's realm.
info: |
  GeneratorFunction ( p1, p2, … , pn, body )

  [...]
  3. Return ? CreateDynamicFunction(C, NewTarget, generator, args).

  CreateDynamicFunction ( constructor, newTarget, kind, args )

  [...]
  18. Let proto be ? GetPrototypeFromConstructor(newTarget, fallbackProto).
  19. Let realmF be the current Realm Record.
  20. Let scope be realmF.[[GlobalEnv]].
  21. Let F be ! OrdinaryFunctionCreate(proto, parameters, body, non-lexical-this, scope).
  [...]
  23. If kind is generator, then
    a. Let prototype be OrdinaryObjectCreate(%Generator.prototype%).
    b. Perform DefinePropertyOrThrow(F, "prototype", PropertyDescriptor { [[Value]]: prototype,
    [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: false }).
  [...]
  30. Return F.
features: [generators, cross-realm, Reflect]
---*/

var realmA = $262.createRealm().global;
realmA.calls = 0;
var aGeneratorFunction = realmA.eval("(function* () {})").constructor;
var aGeneratorPrototype = Object.getPrototypeOf(
  realmA.eval("(function* () {})").prototype
);

var realmB = $262.createRealm().global;
var bGeneratorFunction = realmB.eval("(function* () {})").constructor;
var newTarget = new realmB.Function();
newTarget.prototype = null;

var fn = Reflect.construct(aGeneratorFunction, ["calls += 1;"], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), bGeneratorFunction.prototype);
assert.sameValue(Object.getPrototypeOf(fn.prototype), aGeneratorPrototype);

var gen = fn();
assert(gen instanceof realmA.Object);
gen.next();
assert.sameValue(realmA.calls, 1);
