// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-ecmascript-function-objects-construct-argumentslist-newtarget
description: Error retrieving function realm from revoked Proxy exotic object
info: |
  [...]
  5. If kind is "base", then
     a. Let thisArgument be ? OrdinaryCreateFromConstructor(newTarget,
        "%ObjectPrototype%").
  [...]

  9.1.13 OrdinaryCreateFromConstructor

  [...]
  2. Let proto be ? GetPrototypeFromConstructor(constructor,
     intrinsicDefaultProto).
  [...]

  9.1.14 GetPrototypeFromConstructor

  [...]
  3. Let proto be ? Get(constructor, "prototype").
  4. If Type(proto) is not Object, then
     a. Let realm be ? GetFunctionRealm(constructor).

  7.3.22 GetFunctionRealm

  [...]
  2. If obj has a [[Realm]] internal slot, then
     [...]
  3. If obj is a Bound Function exotic object, then
     [...]
  4. If obj is a Proxy exotic object, then
     a. If the value of the [[ProxyHandler]] internal slot of obj is null,
        throw a TypeError exception.
features: [Proxy]
---*/

// Defer proxy revocation until after the `constructor` property has been
// accessed
var handlers = {
  get: function() {
    handle.revoke();
  }
};
var handle = Proxy.revocable(function() {}, handlers);
var f = handle.proxy;

assert.sameValue(typeof f, 'function');

assert.throws(TypeError, function() {
  new f();
});
