// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/

// BigInt literals as property keys.
{
  let o = {
    0n: "0",
    1n: "1",

    // 2**31
    2147483647n: "2^31-1",
    2147483648n: "2^31",
    2147483649n: "2^31+1",

    // 2**32
    4294967295n: "2^32-1",
    4294967296n: "2^32",
    4294967297n: "2^32+1",

    // 2n**63n
    9223372036854775807n: "2^63-1",
    9223372036854775808n: "2^63",
    9223372036854775809n: "2^63+1",

    // 2n**64n
    18446744073709551615n: "2^64-1",
    18446744073709551616n: "2^64",
    18446744073709551617n: "2^64+1",
  };

  assert.sameValue(o[0], "0");
  assert.sameValue(o[1], "1");

  assert.sameValue(o[2147483647], "2^31-1");
  assert.sameValue(o[2147483648], "2^31");
  assert.sameValue(o[2147483649], "2^31+1");

  assert.sameValue(o[4294967295], "2^32-1");
  assert.sameValue(o[4294967296], "2^32");
  assert.sameValue(o[4294967297], "2^32+1");

  assert.sameValue(o["9223372036854775807"], "2^63-1");
  assert.sameValue(o["9223372036854775808"], "2^63");
  assert.sameValue(o["9223372036854775809"], "2^63+1");

  assert.sameValue(o["18446744073709551615"], "2^64-1");
  assert.sameValue(o["18446744073709551616"], "2^64");
  assert.sameValue(o["18446744073709551617"], "2^64+1");
}

// With non-decimal different base.
{
  let o = {
    0b1n: "1",
    0o2n: "2",
    0x3n: "3",
  };

  assert.sameValue(o[1], "1");
  assert.sameValue(o[2], "2");
  assert.sameValue(o[3], "3");
}

// With numeric separators.
{
  let o = {
    1_2_3n: "123",
  };

  assert.sameValue(o[123], "123");
}

// BigInt literals as method property names.
{
  let o = {
    1n() {},
    *2n() {},
    async 3n() {},
    async* 4n() {},
    get 5n() {},
    set 6n(x) {},
  };

  assert.compareArray(Object.getOwnPropertyNames(o), [
    "1", "2", "3", "4", "5", "6",
  ]);

  assert.sameValue(o[1].name, "1");
  assert.sameValue(o[2].name, "2");
  assert.sameValue(o[3].name, "3");
  assert.sameValue(o[4].name, "4");
  assert.sameValue(Object.getOwnPropertyDescriptor(o, 5).get.name, "get 5");
  assert.sameValue(Object.getOwnPropertyDescriptor(o, 6).set.name, "set 6");
}

// BigInt literals as class method property names.
{
  class C {
   1n() {}
   *2n() {}
   async 3n() {}
   async* 4n() {}
   get 5n() {}
   set 6n(x) {}
  }
  let o = C.prototype;

  assert.compareArray(Object.getOwnPropertyNames(o), [
   "1", "2", "3", "4", "5", "6",
   "constructor",
  ]);

  assert.sameValue(o[1].name, "1");
  assert.sameValue(o[2].name, "2");
  assert.sameValue(o[3].name, "3");
  assert.sameValue(o[4].name, "4");
  assert.sameValue(Object.getOwnPropertyDescriptor(o, 5).get.name, "get 5");
  assert.sameValue(Object.getOwnPropertyDescriptor(o, 6).set.name, "set 6");
}

// BigInt literals as static class method property names.
{
  class C {
   static 1n() {}
   static *2n() {}
   static async 3n() {}
   static async* 4n() {}
   static get 5n() {}
   static set 6n(x) {}
  }
  let o = C;

  // NB: Sort names because lazily resolved "length" and "name" properties are
  // inserted in the wrong order.
  assert.compareArray(Object.getOwnPropertyNames(o).sort(), [
   "1", "2", "3", "4", "5", "6",
   "length", "name", "prototype",
  ]);

  assert.sameValue(o[1].name, "1");
  assert.sameValue(o[2].name, "2");
  assert.sameValue(o[3].name, "3");
  assert.sameValue(o[4].name, "4");
  assert.sameValue(Object.getOwnPropertyDescriptor(o, 5).get.name, "get 5");
  assert.sameValue(Object.getOwnPropertyDescriptor(o, 6).set.name, "set 6");
}

// BigInt literals as class field property names.
{
  let o = new class {
    1n;
    2n = "ok";
  };

  assert.sameValue(o[1], undefined);
  assert.sameValue(o[2], "ok");
}

// In binding destructuring contexts.
{
  let {0n: a} = ["ok"];
  assert.sameValue(a, "ok");
}

// In binding destructuring contexts with object rest pattern.
{
  let {0n: a, ...b} = ["ok", "test"];
  assert.sameValue(a, "ok");
  assert.compareArray(Object.getOwnPropertyNames(b), ["1"]);
}

// In assignment destructuring contexts.
{
  let a;
  ({0n: a} = ["ok"]);
  assert.sameValue(a, "ok");
}

// In assignment destructuring contexts with object rest pattern.
{
  let a, b;
  ({0n: a, ...b} = ["ok", "test"]);
  assert.sameValue(a, "ok");
  assert.compareArray(Object.getOwnPropertyNames(b), ["1"]);
}

// BigInt literals as inferred names.
{
  let o = {
    0xan: function(){},
  };

  assert.sameValue(o[10].name, "10");
}
