// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: SuperClass may be a bound function object
info: |
  ClassDefinitionEvaluation

  [...]
  5. Else,
    [...]
    f. Else if IsConstructor(superclass) is false, throw a TypeError exception.
    g. Else,
      i. Let protoParent be ? Get(superclass, "prototype").
      ii. If Type(protoParent) is neither Object nor Null, throw a TypeError exception.
      iii. Let constructorParent be superclass.
features: [class]
---*/

var bound = function() {}.bind();
bound.prototype = {};

class C extends bound {}

assert.sameValue(Object.getPrototypeOf(new C()), C.prototype);
