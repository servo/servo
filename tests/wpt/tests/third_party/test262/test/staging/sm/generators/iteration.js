// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-generators-shell.js]
flags:
  - noStrict
description: |
  pending
esid: pending
features: [host-gc-required]
---*/
// This file was written by Andy Wingo <wingo@igalia.com> and originally
// contributed to V8 as generators-objects.js, available here:
//
// http://code.google.com/p/v8/source/browse/branches/bleeding_edge/test/mjsunit/harmony/generators-objects.js

// Test aspects of the generator runtime.


var GeneratorFunction = (function*(){yield 1;}).constructor;


function TestGeneratorResultPrototype() {
    function* g() { yield 1; }
    var iter = g();
    assertIteratorNext(iter, 1);
    assertIteratorDone(iter, undefined);
    assertIteratorDone(iter, undefined);
}
TestGeneratorResultPrototype();

function TestGenerator(g, expected_values_for_next,
                       send_val, expected_values_for_send) {
    function testNext(thunk) {
        var iter = thunk();
        for (var i = 0; i < expected_values_for_next.length; i++) {
            assertIteratorResult(iter.next(), expected_values_for_next[i],
                                 i == expected_values_for_next.length - 1);
        }
        assertIteratorDone(iter, undefined);
    }
    function testSend(thunk) {
        var iter = thunk();
        for (var i = 0; i < expected_values_for_send.length; i++) {
            assertIteratorResult(iter.next(send_val),
                                 expected_values_for_send[i],
                                 i == expected_values_for_send.length - 1);
        }
        assertIteratorDone(iter, undefined);
    }
    function testThrow(thunk) {
        for (var i = 0; i < expected_values_for_next.length; i++) {
            var iter = thunk();
            for (var j = 0; j < i; j++) {
                assertIteratorResult(iter.next(),
                                     expected_values_for_next[j],
                                     j == expected_values_for_next.length - 1);
            }
            var Sentinel = function () {}
            assert.throws(Sentinel, function () { iter.throw(new Sentinel); });
            assertIteratorDone(iter, undefined);
        }
    }

    testNext(g);
    testSend(g);
    testThrow(g);

    testNext(function*() { return yield* g(); });
    testSend(function*() { return yield* g(); });
    testThrow(function*() { return yield* g(); });

    if (g instanceof GeneratorFunction) {
        testNext(function() { return g(); });
        testSend(function() { return g(); });
        testThrow(function() { return g(); });
    }
}

TestGenerator(function* g1() { },
              [undefined],
              "foo",
              [undefined]);

TestGenerator(function* g2() { yield 1; },
              [1, undefined],
              "foo",
              [1, undefined]);

TestGenerator(function* g3() { yield 1; yield 2; },
              [1, 2, undefined],
              "foo",
              [1, 2, undefined]);

TestGenerator(function* g4() { yield 1; yield 2; return 3; },
              [1, 2, 3],
              "foo",
              [1, 2, 3]);

TestGenerator(function* g5() { return 1; },
              [1],
              "foo",
              [1]);

TestGenerator(function* g6() { var x = yield 1; return x; },
              [1, undefined],
              "foo",
              [1, "foo"]);

TestGenerator(function* g7() { var x = yield 1; yield 2; return x; },
              [1, 2, undefined],
              "foo",
              [1, 2, "foo"]);

TestGenerator(function* g8() { for (var x = 0; x < 4; x++) { yield x; } },
              [0, 1, 2, 3, undefined],
              "foo",
              [0, 1, 2, 3, undefined]);

// Generator with arguments.
TestGenerator(
    function g9() {
        return (function*(a, b, c, d) {
            yield a; yield b; yield c; yield d;
        })("fee", "fi", "fo", "fum");
    },
    ["fee", "fi", "fo", "fum", undefined],
    "foo",
    ["fee", "fi", "fo", "fum", undefined]);

// Too few arguments.
TestGenerator(
    function g10() {
        return (function*(a, b, c, d) {
            yield a; yield b; yield c; yield d;
        })("fee", "fi");
    },
    ["fee", "fi", undefined, undefined, undefined],
    "foo",
    ["fee", "fi", undefined, undefined, undefined]);

// Too many arguments.
TestGenerator(
    function g11() {
        return (function*(a, b, c, d) {
            yield a; yield b; yield c; yield d;
        })("fee", "fi", "fo", "fum", "I smell the blood of an Englishman");
    },
    ["fee", "fi", "fo", "fum", undefined],
    "foo",
    ["fee", "fi", "fo", "fum", undefined]);

// The arguments object.
TestGenerator(
    function g12() {
        return (function*(a, b, c, d) {
            for (var i = 0; i < arguments.length; i++) {
                yield arguments[i];
            }
        })("fee", "fi", "fo", "fum", "I smell the blood of an Englishman");
    },
    ["fee", "fi", "fo", "fum", "I smell the blood of an Englishman",
     undefined],
    "foo",
    ["fee", "fi", "fo", "fum", "I smell the blood of an Englishman",
     undefined]);

// Access to captured free variables.
TestGenerator(
    function g13() {
        return (function(a, b, c, d) {
            return (function*() {
                yield a; yield b; yield c; yield d;
            })();
        })("fee", "fi", "fo", "fum");
    },
    ["fee", "fi", "fo", "fum", undefined],
    "foo",
    ["fee", "fi", "fo", "fum", undefined]);

// Abusing the arguments object.
TestGenerator(
    function g14() {
        return (function*(a, b, c, d) {
            arguments[0] = "Be he live";
            arguments[1] = "or be he dead";
            arguments[2] = "I'll grind his bones";
            arguments[3] = "to make my bread";
            yield a; yield b; yield c; yield d;
        })("fee", "fi", "fo", "fum");
    },
    ["Be he live", "or be he dead", "I'll grind his bones", "to make my bread",
     undefined],
    "foo",
    ["Be he live", "or be he dead", "I'll grind his bones", "to make my bread",
     undefined]);

// Abusing the arguments object: strict mode.
TestGenerator(
    function g15() {
        return (function*(a, b, c, d) {
            "use strict";
            arguments[0] = "Be he live";
            arguments[1] = "or be he dead";
            arguments[2] = "I'll grind his bones";
            arguments[3] = "to make my bread";
            yield a; yield b; yield c; yield d;
        })("fee", "fi", "fo", "fum");
    },
    ["fee", "fi", "fo", "fum", undefined],
    "foo",
    ["fee", "fi", "fo", "fum", undefined]);

// GC.
TestGenerator(function* g16() { yield "baz"; $262.gc(); yield "qux"; },
              ["baz", "qux", undefined],
              "foo",
              ["baz", "qux", undefined]);

// Receivers.
TestGenerator(
    function g17() {
        function* g() { yield this.x; yield this.y; }
        var o = { start: g, x: 1, y: 2 };
        return o.start();
    },
    [1, 2, undefined],
    "foo",
    [1, 2, undefined]);

TestGenerator(
    function* g19() {
        var x = 1;
        yield x;
        with({x:2}) { yield x; }
        yield x;
    },
    [1, 2, 1, undefined],
    "foo",
    [1, 2, 1, undefined]);

TestGenerator(
    function* g20() { yield (1 + (yield 2) + 3); },
    [2, NaN, undefined],
    "foo",
    [2, "1foo3", undefined]);

TestGenerator(
    function* g21() { return (1 + (yield 2) + 3); },
    [2, NaN],
    "foo",
    [2, "1foo3"]);

TestGenerator(
    function* g22() { yield (1 + (yield 2) + 3); yield (4 + (yield 5) + 6); },
    [2, NaN, 5, NaN, undefined],
    "foo",
    [2, "1foo3", 5, "4foo6", undefined]);

TestGenerator(
    function* g23() {
        return (yield (1 + (yield 2) + 3)) + (yield (4 + (yield 5) + 6));
    },
    [2, NaN, 5, NaN, NaN],
    "foo",
    [2, "1foo3", 5, "4foo6", "foofoo"]);

// Rewind a try context with and without operands on the stack.
TestGenerator(
    function* g24() {
        try {
            return (yield (1 + (yield 2) + 3)) + (yield (4 + (yield 5) + 6));
        } catch (e) {
            throw e;
        }
    },
    [2, NaN, 5, NaN, NaN],
    "foo",
    [2, "1foo3", 5, "4foo6", "foofoo"]);

// Yielding in a catch context, with and without operands on the stack.
TestGenerator(
    function* g25() {
        try {
            throw (yield (1 + (yield 2) + 3))
        } catch (e) {
            if (typeof e == 'object') throw e;
            return e + (yield (4 + (yield 5) + 6));
        }
    },
    [2, NaN, 5, NaN, NaN],
    "foo",
    [2, "1foo3", 5, "4foo6", "foofoo"]);

// Generator function instances.
TestGenerator(GeneratorFunction(),
              [undefined],
              "foo",
              [undefined]);

TestGenerator(new GeneratorFunction(),
              [undefined],
              "foo",
              [undefined]);

TestGenerator(GeneratorFunction('yield 1;'),
              [1, undefined],
              "foo",
              [1, undefined]);

TestGenerator(
    function() { return GeneratorFunction('x', 'y', 'yield x + y;')(1, 2) },
    [3, undefined],
    "foo",
    [3, undefined]);

// Access to this with formal arguments.
TestGenerator(
    function () {
        return ({ x: 42, g: function* (a) { yield this.x } }).g(0);
    },
    [42, undefined],
    "foo",
    [42, undefined]);

function TestTryCatch(instantiate) {
    function* g() { yield 1; try { yield 2; } catch (e) { yield e; } yield 3; }
    function Sentinel() {}

    function Test1(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        assertIteratorNext(iter, 3);
        assertIteratorDone(iter, undefined);
        assertIteratorDone(iter, undefined);
    }
    Test1(instantiate(g));

    function Test2(iter) {
        assert.throws(Sentinel, function() { iter.throw(new Sentinel); });
        assertIteratorDone(iter, undefined);
    }
    Test2(instantiate(g));

    function Test3(iter) {
        assertIteratorNext(iter, 1);
        assert.throws(Sentinel, function() { iter.throw(new Sentinel); });
        assertIteratorDone(iter, undefined);
    }
    Test3(instantiate(g));

    function Test4(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        var exn = new Sentinel;
        assertIteratorResult(iter.throw(exn), exn, false);
        assertIteratorNext(iter, 3);
        assertIteratorDone(iter, undefined);
        assertIteratorDone(iter, undefined);
    }
    Test4(instantiate(g));

    function Test5(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        var exn = new Sentinel;
        assertIteratorResult(iter.throw(exn), exn, false);
        assertIteratorNext(iter, 3);
        assert.throws(Sentinel, function() { iter.throw(new Sentinel); });
        assertIteratorDone(iter, undefined);

    }
    Test5(instantiate(g));

    function Test6(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        var exn = new Sentinel;
        assertIteratorResult(iter.throw(exn), exn, false);
        assert.throws(Sentinel, function() { iter.throw(new Sentinel); });
        assertIteratorDone(iter, undefined);
    }
    Test6(instantiate(g));
}
TestTryCatch(function (g) { return g(); });
TestTryCatch(function* (g) { return yield* g(); });

function TestTryFinally(instantiate) {
    function* g() { yield 1; try { yield 2; } finally { yield 3; } yield 4; }
    function Sentinel() {}
    function Sentinel2() {}

    function Test1(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        assertIteratorNext(iter, 3);
        assertIteratorNext(iter, 4);
        assertIteratorDone(iter, undefined);
        assertIteratorDone(iter, undefined);
    }
    Test1(instantiate(g));

    function Test2(iter) {
        assert.throws(Sentinel, function() { iter.throw(new Sentinel); });
        assertIteratorDone(iter, undefined);
    }
    Test2(instantiate(g));

    function Test3(iter) {
        assertIteratorNext(iter, 1);
        assert.throws(Sentinel, function() { iter.throw(new Sentinel); });
        assertIteratorDone(iter, undefined);
    }
    Test3(instantiate(g));

    function Test4(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        assertIteratorResult(iter.throw(new Sentinel), 3, false);
        assert.throws(Sentinel, function() { iter.next(); });
        assertIteratorDone(iter, undefined);

    }
    Test4(instantiate(g));

    function Test5(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        assertIteratorResult(iter.throw(new Sentinel), 3, false);
        assert.throws(Sentinel2, function() { iter.throw(new Sentinel2); });
        assertIteratorDone(iter, undefined);
    }
    Test5(instantiate(g));

    function Test6(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        assertIteratorNext(iter, 3);
        assert.throws(Sentinel, function() { iter.throw(new Sentinel); });
        assertIteratorDone(iter, undefined);
    }
    Test6(instantiate(g));

    function Test7(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        assertIteratorNext(iter, 3);
        assertIteratorNext(iter, 4);
        assert.throws(Sentinel, function() { iter.throw(new Sentinel); });
        assertIteratorDone(iter, undefined);
    }
    Test7(instantiate(g));
}
TestTryFinally(function (g) { return g(); });
TestTryFinally(function* (g) { return yield* g(); });

function TestNestedTry(instantiate) {
    function* g() {
        try {
            yield 1;
            try { yield 2; } catch (e) { yield e; }
            yield 3;
        } finally {
            yield 4;
        }
        yield 5;
    }
    function Sentinel() {}
    function Sentinel2() {}

    function Test1(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        assertIteratorNext(iter, 3);
        assertIteratorNext(iter, 4);
        assertIteratorNext(iter, 5);
        assertIteratorDone(iter, undefined);
        assertIteratorDone(iter, undefined);
    }
    Test1(instantiate(g));

    function Test2(iter) {
        assert.throws(Sentinel, function() { iter.throw(new Sentinel); });
        assertIteratorDone(iter, undefined);
    }
    Test2(instantiate(g));

    function Test3(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorResult(iter.throw(new Sentinel), 4, false);
        assert.throws(Sentinel, function() { iter.next(); });
        assertIteratorDone(iter, undefined);
    }
    Test3(instantiate(g));

    function Test4(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorResult(iter.throw(new Sentinel), 4, false);
        assert.throws(Sentinel2, function() { iter.throw(new Sentinel2); });
        assertIteratorDone(iter, undefined);
    }
    Test4(instantiate(g));

    function Test5(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        var exn = new Sentinel;
        assertIteratorResult(iter.throw(exn), exn, false);
        assertIteratorNext(iter, 3);
        assertIteratorNext(iter, 4);
        assertIteratorNext(iter, 5);
        assertIteratorDone(iter, undefined);
        assertIteratorDone(iter, undefined);

    }
    Test5(instantiate(g));

    function Test6(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        var exn = new Sentinel;
        assertIteratorResult(iter.throw(exn), exn, false);
        assertIteratorResult(iter.throw(new Sentinel2), 4, false);
        assert.throws(Sentinel2, function() { iter.next(); });
        assertIteratorDone(iter, undefined);
    }
    Test6(instantiate(g));

    function Test7(iter) {
        assertIteratorNext(iter, 1);
        assertIteratorNext(iter, 2);
        var exn = new Sentinel;
        assertIteratorResult(iter.throw(exn), exn, false);
        assertIteratorNext(iter, 3);
        assertIteratorResult(iter.throw(new Sentinel2), 4, false);
        assert.throws(Sentinel2, function() { iter.next(); });
        assertIteratorDone(iter, undefined);

    }
    Test7(instantiate(g));

    // That's probably enough.
}
TestNestedTry(function (g) { return g(); });
TestNestedTry(function* (g) { return yield* g(); });

function TestRecursion() {
    function TestNextRecursion() {
        function* g() { yield iter.next(); }
        var iter = g();
        return iter.next();
    }
    function TestSendRecursion() {
        function* g() { yield iter.next(42); }
        var iter = g();
        return iter.next();
    }
    function TestThrowRecursion() {
        function* g() { yield iter.throw(1); }
        var iter = g();
        return iter.next();
    }
    assert.throws(TypeError, TestNextRecursion);
    assert.throws(TypeError, TestSendRecursion);
    assert.throws(TypeError, TestThrowRecursion);
}
TestRecursion();

