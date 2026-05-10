// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
var obj = {
  ["func"]: function() {},
  ["genFunc"]: function*() {},
  ["asyncFunc"]: async function() {},
  ["asyncGenFunc"]: async function*() {},
  ["arrowFunc"]: ()=>{},
  ["asyncArrowFunc"]: async ()=>{},
  ["method"]() {},
  ["anonClass"]: class {},
  ["nonAnonymousFunc"]: function F() {},
  ["nonAnonymousClass"]: class C{},
  get ["getter"]() {},
  set ["setter"](x) {},
};

assert.sameValue(obj.func.name, "func");
assert.sameValue(obj.genFunc.name, "genFunc");
assert.sameValue(obj.asyncFunc.name, "asyncFunc");
assert.sameValue(obj.asyncGenFunc.name, "asyncGenFunc");
assert.sameValue(obj.arrowFunc.name, "arrowFunc");
assert.sameValue(obj.asyncArrowFunc.name, "asyncArrowFunc");
assert.sameValue(obj.method.name, "method");
assert.sameValue(obj.anonClass.name, "anonClass");
assert.sameValue(obj.nonAnonymousFunc.name, "F");
assert.sameValue(obj.nonAnonymousClass.name, "C");

assert.sameValue(Object.getOwnPropertyDescriptor(obj, "getter").get.name, "get getter");
assert.sameValue(Object.getOwnPropertyDescriptor(obj, "setter").set.name, "set setter");

let dummy = class {
  ["func"]() {}
  *["genFunc"] () {}
  async ["asyncFunc"]() {}
  async *["asyncGenFunc"]() {}
  ["arrowFunc"] = ()=>{}
  ["asyncArrowFunc"] = async ()=>{};
  ["method"]() {}
  get ["getter"]() {}
  set ["setter"](x) {}
};

let dum = new dummy();

assert.sameValue(dum.func.name, "func");
assert.sameValue(dum.genFunc.name, "genFunc");
assert.sameValue(dum.asyncFunc.name, "asyncFunc");
assert.sameValue(dum.asyncGenFunc.name, "asyncGenFunc");
assert.sameValue(dum.arrowFunc.name, "arrowFunc");
assert.sameValue(dum.asyncArrowFunc.name, "asyncArrowFunc");
assert.sameValue(dum.method.name, "method");

assert.sameValue(Object.getOwnPropertyDescriptor(dummy.prototype, "getter").get.name, "get getter");
assert.sameValue(Object.getOwnPropertyDescriptor(dummy.prototype, "setter").set.name, "set setter");

