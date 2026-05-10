// Copyright (C) 2009 the Sputnik authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.prototype.isprototypeof
description: >
  Object.prototype.isPrototypeOf returns true if either parameter V
  and O refer to the same object or O is in [[Prototype]] chain of V.
info: |
  Object.prototype.isPrototypeOf ( V )

  ...
  3. Repeat,
    a. Set V to ? V.[[GetPrototypeOf]]().
    b. If V is null, return false.
    c. If SameValue(O, V) is true, return true.
---*/

function USER_FACTORY(name) {
  this.name = name;
  this.getName = function() {
    return name;
  };
}

function FORCEDUSER_FACTORY(name, grade) {
  this.name = name;
  this.grade = grade;
  this.getGrade = function() {
    return grade;
  };
}

var proto = new USER_FACTORY("noname");

FORCEDUSER_FACTORY.prototype = proto;

var luke = new FORCEDUSER_FACTORY("Luke Skywalker", 12);

assert.sameValue(proto.isPrototypeOf(luke), true);
assert.sameValue(USER_FACTORY.prototype.isPrototypeOf(luke), true);
assert.sameValue(Number.isPrototypeOf(luke), false);
