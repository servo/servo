// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: "#constructor is a valid property name for a public field"
esid: sec-class-definitions-static-semantics-early-errors
features: [class, class-fields-public]
info: |
    ClassElementName : PrivateName;

    It is a Syntax  Error if StringValue of PrivateName is "#constructor".
includes: [propertyHelper.js]
---*/

class C1 {
  ["#constructor"];
}

var c1 = new C1();

assert.sameValue(Object.prototype.hasOwnProperty.call(C1, '#constructor'), false);
verifyProperty(c1, '#constructor', {
  value: undefined,
  configurable: true,
  enumerable: true,
  writable: true,
});

class C2 {
  ["#constructor"] = 42;
}

var c2 = new C2();

assert.sameValue(Object.prototype.hasOwnProperty.call(C2, '#constructor'), false);
verifyProperty(c2, '#constructor', {
  value: 42,
  configurable: true,
  enumerable: true,
  writable: true,
});
