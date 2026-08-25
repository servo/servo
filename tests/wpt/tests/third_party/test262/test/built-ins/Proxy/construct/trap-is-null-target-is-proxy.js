// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-construct-argumentslist-newtarget
description: >
  If "construct" trap is null or undefined, [[Construct]] call is
  properly forwarded to [[ProxyTarget]] (which is also a Proxy object).
info: |
  [[Construct]] ( argumentsList, newTarget )

  [...]
  4. Let target be O.[[ProxyTarget]].
  5. Assert: IsConstructor(target) is true.
  6. Let trap be ? GetMethod(handler, "construct").
  7. If trap is undefined, then
    a. Return ? Construct(target, argumentsList, newTarget).
features: [class, Proxy, Reflect, Reflect.construct]
---*/

class Foo {
  constructor(arg) {
    this.arg = arg;
  } 
}

var FooTarget = new Proxy(Foo, {});
var FooProxy = new Proxy(FooTarget, {
  construct: null,
});

var foo = new FooProxy(1);
assert(foo instanceof Foo);
assert.sameValue(foo.arg, 1);

class Bar extends Foo {
  get isBar() {
    return true;
  }
}

var bar = Reflect.construct(FooProxy, [2], Bar);
assert(bar instanceof Bar);
assert.sameValue(bar.arg, 2);
assert(bar.isBar);
