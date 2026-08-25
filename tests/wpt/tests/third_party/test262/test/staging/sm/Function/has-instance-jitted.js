// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
const OriginalHasInstance = Function.prototype[Symbol.hasInstance];

// Ensure that folding doesn't impact user defined @@hasInstance methods.
{
    function Test() {
        this.x = 1;
    }

    Object.defineProperty(Test, Symbol.hasInstance,
                          {writable: true, value: () => false});

    function x(t) {
        return t instanceof Test;
    }

    function y() {
        let t = new Test;
        let b = true;
        for (let i = 0; i < 10; i++) {
            b = b && x(t);
        }
        return b;
    }


    function z() {
        let f = 0;
        let t = 0;
        for (let i = 0; i < 100; i++)
            assert.sameValue(y(), false);
    }

    z();
}

// Ensure that the jitting does not clobber user defined @@hasInstance methods.
{
    function a() {
        function b() {};
        b.__proto__ = a.prototype;
        return b;
    };
    let c = new a();

    let t = 0;
    let f = 0;
    let e = 0;
    for (let i = 0; i < 40000; i++) {
        if (i == 20000)
            Object.defineProperty(a.prototype, Symbol.hasInstance,
                                  {writable: true, value: () => true});
        if (i == 30000)
            Object.setPrototypeOf(c, Function.prototype);

        if (1 instanceof c) {
            t++;
        } else {
            f++;
        }
    }

    assert.sameValue(t, 10000);
    assert.sameValue(f, 30000);
}

{
    function a() {};
    function b() {};
    Object.defineProperty(a, Symbol.hasInstance, {writable: true, value: () => true});
    assert.sameValue(b instanceof a, true);
    for (let _ of Array(10000))
        assert.sameValue(b instanceof a, true);
}

{
    function a(){};
    function b(){};
    function c(){};
    function d(){};
    function e(){};
    Object.defineProperty(a, Symbol.hasInstance, {value: () => true });
    Object.defineProperty(b, Symbol.hasInstance, {value: () => true });
    Object.defineProperty(c, Symbol.hasInstance, {value: () => true });
    Object.defineProperty(d, Symbol.hasInstance, {value: () => true });
    let funcs = [a, b, c, d];
    for (let f of funcs)
        assert.sameValue(e instanceof f, true);

    for (let _ of Array(10001)) {
        for (let f of funcs)
            assert.sameValue(e instanceof f, true);
    }
}

