// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/assertThrowsValue.js]
description: |
  pending
esid: pending
---*/
// Errors accessing next, done, or value don't cause an exception to be
// thrown into the iterator of a yield*.

function* g(n) { for (var i=0; i<n; i++) yield i; }
function* delegate(iter) { return yield* iter; }

var log = "", inner, outer;

// That var is poisoooooon, p-poison poison...
var Poison = new Error;

function log_calls(method) {
    return function () {
        log += "x"
        return method.call(this);
    }
}

function poison(receiver, prop) {
    Object.defineProperty(receiver, prop, { get: function () { throw Poison } });
}

// Poison inner.next.
inner = g(10);
outer = delegate(inner);
inner.throw = log_calls(inner.throw);
poison(inner, 'next')
assertThrowsValue(outer.next.bind(outer), Poison);
assert.sameValue(log, "");

// Poison result value from inner.
inner = g(10);
outer = delegate(inner);
inner.next = function () { return { done: true, get value() { throw Poison} } };
inner.throw = log_calls(inner.throw);
assertThrowsValue(outer.next.bind(outer), Poison);
assert.sameValue(log, "");

// Poison result done from inner.
inner = g(10);
outer = delegate(inner);
inner.next = function () { return { get done() { throw Poison }, value: 42 } };
inner.throw = log_calls(inner.throw);
assertThrowsValue(outer.next.bind(outer), Poison);
assert.sameValue(log, "");

// mischief managed.
