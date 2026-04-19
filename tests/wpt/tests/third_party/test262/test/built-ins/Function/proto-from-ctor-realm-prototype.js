// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdynamicfunction
description: >
  While default [[Prototype]] value derives from realm of the newTarget,
  "prototype" object inherits from %Object.prototype% of constructor's realm.
info: |
  Function ( p1, p2, … , pn, body )

  [...]
  3. Return ? CreateDynamicFunction(C, NewTarget, normal, args).

  CreateDynamicFunction ( constructor, newTarget, kind, args )

  [...]
  18. Let proto be ? GetPrototypeFromConstructor(newTarget, fallbackProto).
  19. Let realmF be the current Realm Record.
  20. Let scope be realmF.[[GlobalEnv]].
  21. Let F be ! OrdinaryFunctionCreate(proto, parameters, body, non-lexical-this, scope).
  [...]
  25. Else if kind is normal, perform MakeConstructor(F).
  [...]
  30. Return F.

  MakeConstructor ( F [ , writablePrototype [ , prototype ] ] )

  [...]
  7. If prototype is not present, then
    a. Set prototype to OrdinaryObjectCreate(%Object.prototype%).
    [...]
  8. Perform ! DefinePropertyOrThrow(F, "prototype", PropertyDescriptor {[[Value]]: prototype,
  [[Writable]]: writablePrototype, [[Enumerable]]: false, [[Configurable]]: false }).
features: [cross-realm, Reflect]
---*/

var realmA = $262.createRealm().global;
realmA.calls = 0;

var realmB = $262.createRealm().global;
var newTarget = new realmB.Function();
newTarget.prototype = null;

var fn = Reflect.construct(realmA.Function, ["calls += 1;"], newTarget);
assert.sameValue(Object.getPrototypeOf(fn), realmB.Function.prototype);
assert.sameValue(Object.getPrototypeOf(fn.prototype), realmA.Object.prototype);

assert(new fn() instanceof realmA.Object);
assert.sameValue(realmA.calls, 1);
