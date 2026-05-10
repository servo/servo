// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
---*/
// Test that each yield* loop just checks "done", and "value" is only
// fetched once at the end.

var log = "";

function collect_results(iter) {
    var ret = [];
    var result;
    do {
        result = iter.next();
        ret.push(result);
    } while (!result.done);
    return ret;
}

function Iter(val, count) {
    function next() {
        log += 'n';
        return {
            get done() { log += "d"; return count-- == 0; },
            get value() { log += "v"; return val; }
        }
    }

    function iterator() {
        log += 'i';
        return this;
    }

    this.next = next;
    this[Symbol.iterator] = iterator;
}

function* delegate(iter) { return yield* iter; }

var inner = new Iter(42, 5);
var outer = delegate(inner);

// Five values, and one terminal value.
outer.next();
outer.next();
outer.next();
outer.next();
outer.next();
outer.next();

assert.sameValue(log, "indndndndndndv");

// Outer's dead, man.  Outer's dead.
assert.deepEqual(outer.next(), {value: undefined, done: true});

// No more checking the iterator.
assert.sameValue(log, "indndndndndndv");

