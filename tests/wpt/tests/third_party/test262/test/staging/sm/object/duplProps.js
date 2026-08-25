// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
/*
 * ES6 allows duplicate property names in object literals, even in strict mode.
 * These tests modify the tests in test262 to reflect this change.
 */

var a;

// test262/ch11/11.1/11.1.5/11.1.5-4-4-a-1-s.js
a = function() { "use strict"; return { foo: 0, foo : 1 }};
assert.sameValue(a().foo, 1);
a = function() { return { foo: 0, foo : 1 }};
assert.sameValue(a().foo, 1);

// test262/ch11/11.1/11.1.5/11.1.5_4-4-b-1.js
a = function() { "use strict"; return { foo : 1, get foo() { return 2; }}};
assert.sameValue(a().foo, 2);
a = function() { return { foo : 1, get foo() { return 2;} }};
assert.sameValue(a().foo, 2);

// test262/ch11/11.1/11.1.5/11.1.5_4-4-c-1.js
a = function() { "use strict"; return { get foo() { return 2; }, foo : 1 }};
assert.sameValue(a().foo, 1);
a = function() { return { get foo() { return 2; }, foo : 1 }};
assert.sameValue(a().foo, 1);

// test262/ch11/11.1/11.1.5/11.1.5_4-4-b-2.js
a = function() { "use strict"; return { foo : 1, set foo(a) { throw 2; }}};
try {
    a().foo = 5;
    throw new Error("2 should be thrown here");
} catch (e) {
    if (e !== 2)
        throw new Error("2 should be thrown here");
}
a = function() { return { foo : 1, set foo(a) { throw 2;} }};
try {
    a().foo = 5;
    throw new Error("2 should be thrown here");
} catch (e) {
    if (e !== 2)
        throw new Error("2 should be thrown here");
}

// test262/ch11/11.1/11.1.5/11.1.5_4-4-d-1.js
a = function() { "use strict"; return { get foo() { return 2; }, get foo() { return 3; } }};
assert.sameValue(a().foo, 3);
a = function() { return { get foo() { return 2; }, get foo() { return 3; } }};
assert.sameValue(a().foo, 3);

// test262/ch11/11.1/11.1.5/11.1.5_4-4-c-2.js
a = function() { "use strict"; return { set foo(a) { throw 2; }, foo : 1 }};
assert.sameValue(a().foo, 1);
a = function() { return { set foo(a) { throw 2; }, foo : 1 }};
assert.sameValue(a().foo, 1);

// test262/ch11/11.1/11.1.5/11.1.5_4-4-d-2.js
a = function() { "use strict"; return { set foo(a) { throw 2; }, set foo(a) { throw 3; }}};
try {
    a().foo = 5;
    throw new Error("3 should be thrown here");
} catch (e) {
    if (e !== 3)
        throw new Error("3 should be thrown here");
}
a = function() { return { set foo(a) { throw 2; }, set foo(a) { throw 3; }}};
try {
    a().foo = 5;
    throw new Error("3 should be thrown here");
} catch (e) {
    if (e !== 3)
        throw new Error("3 should be thrown here");
}

// test262/ch11/11.1/11.1.5/11.1.5_4-4-d-3.js
a = function() { "use strict"; return { get foo() { return 2; }, set foo(a) { throw 3; },
            get foo() { return 4; }}};
try {
    assert.sameValue(a().foo, 4);
    a().foo = 5;
    throw new Error("3 should be thrown here");
} catch (e) {
    if (e !== 3)
        throw new Error("3 should be thrown here");
}
a = function() { return { get foo() { return 2; }, set foo(a) { throw 3; },
            get foo() { return 4; }}};
try {
    assert.sameValue(a().foo, 4);
    a().foo = 5;
    throw new Error("3 should be thrown here");
} catch (e) {
    if (e !== 3)
        throw new Error("3 should be thrown here");
}

// test262/ch11/11.1/11.1.5/11.1.5_4-4-d-4.js
a = function() { "use strict"; return { set foo(a) { throw 2; }, get foo() { return 4; },
            set foo(a) { throw 3; }}};
try {
    assert.sameValue(a().foo, 4);
    a().foo = 5;
    throw new Error("3 should be thrown here");
} catch (e) {
    if (e !== 3)
        throw new Error("3 should be thrown here");
}
a = function() { return { set foo(a) { throw 2; }, get foo() { return 4; },
            set foo(a) { throw 3; }}};
try {
    assert.sameValue(a().foo, 4);
    a().foo = 5;
    throw new Error("3 should be thrown here");
} catch (e) {
    if (e !== 3)
        throw new Error("3 should be thrown here");
}

