// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  iterator.next() should not be called when after iterator completes
info: bugzilla.mozilla.org/show_bug.cgi?id=1184922
esid: pending
---*/

var log;
function reset() {
    log = "";
}
var obj = new Proxy({}, {
    set(that, name, value) {
        var v;
        if (value instanceof Function || value instanceof RegExp)
            v = value.toString();
        else
            v = JSON.stringify(value);
        log += "set:" + name + "=" + v + ",";
        return true;
    }
});
function createIterable(n) {
    return {
        i: 0,
        [Symbol.iterator]() {
            return this;
        },
        next() {
            log += "next,";
            this.i++;
            if (this.i <= n)
                return {value: this.i, done: false};
            return {value: 0, done: true};
        }
    };
}

// Simple pattern.

reset();
[obj.a, obj.b, obj.c] = createIterable(0);
assert.sameValue(log,
         "next," +
         "set:a=undefined," +
         "set:b=undefined," +
         "set:c=undefined,");

reset();
[obj.a, obj.b, obj.c] = createIterable(1);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "set:b=undefined," +
         "set:c=undefined,");

reset();
[obj.a, obj.b, obj.c] = createIterable(2);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "set:b=2," +
         "next," +
         "set:c=undefined,");

reset();
[obj.a, obj.b, obj.c] = createIterable(3);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "set:b=2," +
         "next," +
         "set:c=3,");

// Elision.

reset();
[obj.a, , obj.b, , , obj.c, ,] = createIterable(0);
assert.sameValue(log,
         "next," +
         "set:a=undefined," +
         "set:b=undefined," +
         "set:c=undefined,");

reset();
[obj.a, , obj.b, , , obj.c, ,] = createIterable(1);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "set:b=undefined," +
         "set:c=undefined,");

reset();
[obj.a, , obj.b, , , obj.c, ,] = createIterable(2);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "next," +
         "set:b=undefined," +
         "set:c=undefined,");

reset();
[obj.a, , obj.b, , , obj.c, ,] = createIterable(3);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "next," +
         "set:b=3," +
         "next," +
         "set:c=undefined,");

reset();
[obj.a, , obj.b, , , obj.c, ,] = createIterable(4);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "next," +
         "set:b=3," +
         "next," +
         "next," +
         "set:c=undefined,");

reset();
[obj.a, , obj.b, , , obj.c, ,] = createIterable(5);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "next," +
         "set:b=3," +
         "next," +
         "next," +
         "next," +
         "set:c=undefined,");

reset();
[obj.a, , obj.b, , , obj.c, ,] = createIterable(6);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "next," +
         "set:b=3," +
         "next," +
         "next," +
         "next," +
         "set:c=6," +
         "next,");

reset();
[obj.a, , obj.b, , , obj.c, ,] = createIterable(7);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "next," +
         "set:b=3," +
         "next," +
         "next," +
         "next," +
         "set:c=6," +
         "next,");

// Rest.

reset();
[...obj.r] = createIterable(0);
assert.sameValue(log,
         "next," +
         "set:r=[],");

reset();
[...obj.r] = createIterable(1);
assert.sameValue(log,
         "next," +
         "next," +
         "set:r=[1],");

reset();
[obj.a, ...obj.r] = createIterable(0);
assert.sameValue(log,
         "next," +
         "set:a=undefined," +
         "set:r=[],");

reset();
[obj.a, ...obj.r] = createIterable(1);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "set:r=[],");

reset();
[obj.a, ...obj.r] = createIterable(2);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "next," +
         "set:r=[2],");

reset();
[obj.a, obj.b, ...obj.r] = createIterable(0);
assert.sameValue(log,
         "next," +
         "set:a=undefined," +
         "set:b=undefined," +
         "set:r=[],");

reset();
[obj.a, obj.b, ...obj.r] = createIterable(1);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "set:b=undefined," +
         "set:r=[],");

reset();
[obj.a, obj.b, ...obj.r] = createIterable(2);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "set:b=2," +
         "next," +
         "set:r=[],");

reset();
[obj.a, obj.b, ...obj.r] = createIterable(3);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "set:b=2," +
         "next," +
         "next," +
         "set:r=[3],");

// Rest and elision.

reset();
[, ...obj.r] = createIterable(0);
assert.sameValue(log,
         "next," +
         "set:r=[],");

reset();
[, ...obj.r] = createIterable(1);
assert.sameValue(log,
         "next," +
         "next," +
         "set:r=[],");

reset();
[, ...obj.r] = createIterable(2);
assert.sameValue(log,
         "next," +
         "next," +
         "next," +
         "set:r=[2],");

reset();
[obj.a, obj.b, , ...obj.r] = createIterable(0);
assert.sameValue(log,
         "next," +
         "set:a=undefined," +
         "set:b=undefined," +
         "set:r=[],");

reset();
[obj.a, obj.b, , ...obj.r] = createIterable(1);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "set:b=undefined," +
         "set:r=[],");

reset();
[obj.a, obj.b, , ...obj.r] = createIterable(2);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "set:b=2," +
         "next," +
         "set:r=[],");

reset();
[obj.a, obj.b, , ...obj.r] = createIterable(3);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "set:b=2," +
         "next," +
         "next," +
         "set:r=[],");

reset();
[obj.a, obj.b, , ...obj.r] = createIterable(4);
assert.sameValue(log,
         "next," +
         "set:a=1," +
         "next," +
         "set:b=2," +
         "next," +
         "next," +
         "next," +
         "set:r=[4],");
