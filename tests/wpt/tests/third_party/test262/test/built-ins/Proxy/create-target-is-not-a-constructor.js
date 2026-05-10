// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxycreate
description: >
    A Proxy exotic object only accepts a constructor call if target is
    constructor.
info: |
    ProxyCreate ( target, handler )

    If IsCallable(target) is true, then
        Set P.[[Call]] as specified in 9.5.12.
        If IsConstructor(target) is true, then
            Set P.[[Construct]] as specified in 9.5.13.
    ...

    Runtime Semantics: EvaluateNew(constructProduction, arguments)

    8. If IsConstructor (constructor) is false, throw a TypeError exception.
includes: [isConstructor.js]
features: [Proxy, Reflect.construct, arrow-function]
---*/

var proxy = new Proxy(eval, {});

proxy(); // the Proxy object is callable

assert.sameValue(isConstructor(proxy), false, 'isConstructor(proxy) must return false');
assert.throws(TypeError, () => {
  new proxy();
});
