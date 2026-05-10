// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var obj = {
  [1]: function() {},
  [2]: function*() {},
  [3]: async function() {},
  [4]: async function*() {},
  [5]: ()=>{},
  [6]: async ()=>{},
  [7] () {},
  [8]: class {},
  [9]: function F() {},
  [10]: class C{},
  get [11]() {},
  set [12](x) {},
};

assert.sameValue(obj[1].name, "1");
assert.sameValue(obj[2].name, "2");
assert.sameValue(obj[3].name, "3");
assert.sameValue(obj[4].name, "4");
assert.sameValue(obj[5].name, "5");
assert.sameValue(obj[6].name, "6");
assert.sameValue(obj[7].name, "7");
assert.sameValue(obj[8].name, "8");
assert.sameValue(obj[9].name, "F");
assert.sameValue(obj[10].name, "C");
assert.sameValue(Object.getOwnPropertyDescriptor(obj, "11").get.name, "get 11");
assert.sameValue(Object.getOwnPropertyDescriptor(obj, "12").set.name, "set 12");

let dummy = class {
  [1]() {}
  *[2]() {}
  async [3]() {}
  async *[4]() {}
  [5] = ()=>{}
  [6] = async ()=>{};
  [7] () {}
  get [11]() {}
  set [12](x) {}
};

let dum = new dummy();

assert.sameValue(dum[1].name, "1");
assert.sameValue(dum[2].name, "2");
assert.sameValue(dum[3].name, "3");
assert.sameValue(dum[4].name, "4");
assert.sameValue(dum[5].name, "5");
assert.sameValue(dum[6].name, "6");
assert.sameValue(dum[7].name, "7");

assert.sameValue(Object.getOwnPropertyDescriptor(dummy.prototype, "11").get.name, "get 11");
assert.sameValue(Object.getOwnPropertyDescriptor(dummy.prototype, "12").set.name, "set 12");


