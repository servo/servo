// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-defineownproperty-p-desc
description: >
  If "defineProperty" trap is null or undefined, [[DefineOwnProperty]] call
  is properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[DefineOwnProperty]] (P, Desc)

  [...]
  5. Let target be O.[[ProxyTarget]].
  6. Let trap be ? GetMethod(handler, "defineProperty").
  7. If trap is undefined, then
    a. Return ? target.[[DefineOwnProperty]](P, Desc).
features: [Proxy, Reflect]
---*/

var string = new String("str");
var stringTarget = new Proxy(string, {});
var stringProxy = new Proxy(stringTarget, {});

assert(Reflect.defineProperty(stringProxy, "4", {value: 4}));
assert.sameValue(string[4], 4);

assert.throws(TypeError, function() {
  Object.defineProperty(stringProxy, "0", {
    value: "x",
  });
});

Object.preventExtensions(string);
assert(!Reflect.defineProperty(stringProxy, "foo", {value: 5}));


var func = function() {};
var funcTarget = new Proxy(func, {});
var funcProxy = new Proxy(funcTarget, {});

Object.defineProperty(funcProxy, "name", {value: "foo"});
assert.sameValue(func.name, "foo");

assert.throws(TypeError, function() {
  Object.defineProperty(funcProxy, "prototype", {
    set: function(_value) {},
  });
});
