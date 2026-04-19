// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-forin-div-ofbodyevaluation-lhs-stmt-iterator-lhskind-labelset
es6id: 13.7.5.13
description: >
    let ForDeclaration: creates a fresh binding per iteration
---*/

var fns = {};
var obj = Object.create(null);
obj.a = 1;
obj.b = 1;
obj.c = 1;

for (let x in obj) {
  // Store function objects as properties of an object so that their return
  // value may be verified regardless of the for-in statement's enumeration
  // order.
  fns[x] = function() { return x; };
}

assert.sameValue(typeof fns.a, 'function', 'property definition: "a"');
assert.sameValue(fns.a(), 'a');
assert.sameValue(typeof fns.b, 'function', 'property definition: "b"');
assert.sameValue(fns.b(), 'b');
assert.sameValue(typeof fns.c, 'function', 'property definition: "c"');
assert.sameValue(fns.c(), 'c');
