// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
  [[IsHTMLDDA]] object as "prototype" of superclass: `null` check uses strict equality.
info: |
  ClassDefinitionEvaluation

  [...]
  5. Else,
    [...]
    g. Else,
      i. Let protoParent be ? Get(superclass, "prototype").
      ii. If Type(protoParent) is neither Object nor Null, throw a TypeError exception.
      iii. Let constructorParent be superclass.
  6. Let proto be OrdinaryObjectCreate(protoParent).
  [...]
features: [class, IsHTMLDDA]
---*/

function Superclass() {}
Superclass.prototype = $262.IsHTMLDDA;

class C extends Superclass {}
var c = new C();

assert(c instanceof C);
assert(c instanceof Superclass);
