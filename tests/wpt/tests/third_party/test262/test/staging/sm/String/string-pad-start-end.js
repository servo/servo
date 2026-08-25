/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
// `this` must be object coercable.

for (let badThis of [null, undefined]) {
    assert.throws(TypeError, () => {
        String.prototype.padStart.call(badThis, 42, "oups");
    });

    assert.throws(TypeError, () => {
        String.prototype.padEnd.call(badThis, 42, "oups");
    });
}

let proxy = new Proxy({}, {
get(t, name) {
  if (name === Symbol.toPrimitive || name === "toString") return;
  if (name === "valueOf") return () => 42;
  throw "This should not be reachable";
}
});

assert.sameValue("42bloop", String.prototype.padEnd.call(proxy, 7, "bloopie"));

// maxLength must convert to an integer

assert.sameValue("lame", "lame".padStart(0, "foo"));
assert.sameValue("lame", "lame".padStart(0.1119, "foo"));
assert.sameValue("lame", "lame".padStart(-0, "foo"));
assert.sameValue("lame", "lame".padStart(NaN, "foo"));
assert.sameValue("lame", "lame".padStart(-1, "foo"));
assert.sameValue("lame", "lame".padStart({toString: () => 0}, "foo"));

assert.sameValue("lame", "lame".padEnd(0, "foo"));
assert.sameValue("lame", "lame".padEnd(0.1119, "foo"));
assert.sameValue("lame", "lame".padEnd(-0, "foo"));
assert.sameValue("lame", "lame".padEnd(NaN, "foo"));
assert.sameValue("lame", "lame".padEnd(-1, "foo"));
assert.sameValue("lame", "lame".padEnd({toString: () => 0}, "foo"));

assert.throws(TypeError, () => {
    "lame".padStart(Symbol("9900"), 0);
});

assert.throws(TypeError, () => {
    "lame".padEnd(Symbol("9900"), 0);
});

// The fill argument must be string coercable.

assert.sameValue("nulln.", ".".padStart(6, null));
assert.sameValue(".nulln", ".".padEnd(6, null));

assert.sameValue("[obje.", ".".padStart(6, {}));
assert.sameValue(".[obje", ".".padEnd(6, {}));

assert.sameValue("1,2,3.", ".".padStart(6, [1, 2, 3]));
assert.sameValue(".1,2,3", ".".padEnd(6, [1, 2, 3]));

assert.sameValue("aaaaa.", ".".padStart(6, {toString: () => "a"}));
assert.sameValue(".aaaaa", ".".padEnd(6, {toString: () => "a"}));

// undefined is converted to " "

assert.sameValue("     .", ".".padStart(6, undefined));
assert.sameValue(".     ", ".".padEnd(6, undefined));

assert.sameValue("     .", ".".padStart(6));
assert.sameValue(".     ", ".".padEnd(6));

// The empty string has no effect

assert.sameValue("Tilda", "Tilda".padStart(100000, ""));
assert.sameValue("Tilda", "Tilda".padEnd(100000, ""));

assert.sameValue("Tilda", "Tilda".padStart(100000, {toString: () => ""}));
assert.sameValue("Tilda", "Tilda".padEnd(100000, {toString: () => ""}));

// Test repetition against a bruteforce implementation

let filler = "space";
let truncatedFiller = "";
for (let i = 0; i < 2500; i++) {
    truncatedFiller += filler[i % filler.length];
    assert.sameValue(truncatedFiller + "goto", "goto".padStart(5 + i, filler));
    assert.sameValue("goto" + truncatedFiller, "goto".padEnd(5 + i, filler));
}

// [Argument] Length

assert.sameValue(1, String.prototype.padStart.length)
assert.sameValue(1, String.prototype.padEnd.length)

