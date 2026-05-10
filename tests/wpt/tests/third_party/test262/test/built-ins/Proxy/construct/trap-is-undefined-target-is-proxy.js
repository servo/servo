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
  constructor(a, b) {
    this.sum = a + b;
  }
}

var FooBound = Foo.bind(null, 1);
var FooTarget = new Proxy(FooBound, {});
var FooProxy = new Proxy(FooTarget, {
  construct: undefined,
});

var foo = new FooBound(2);
assert(foo instanceof Foo);
assert.sameValue(foo.sum, 3);

class Bar extends Foo {
  get isBar() {
    return true;
  }
}

var bar = Reflect.construct(FooProxy, [3], Bar);
assert(bar instanceof Bar);
assert.sameValue(bar.sum, 4);
assert(bar.isBar);
