// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5.14
description: The Proxy Object is not subclasseable without a prototype
info: |
  14.5.14 Runtime Semantics: ClassDefinitionEvaluation

  5. If ClassHeritageopt is not present, then
    ...
  6. Else
    ...
    e. If superclass is null, then
      ...
    f. Else if IsConstructor(superclass) is false, throw a TypeError exception.
    g. Else
      ...
      ii. Let protoParent be Get(superclass, "prototype").
      iii. ReturnIfAbrupt(protoParent).
      iv. If Type(protoParent) is neither Object nor Null, throw a TypeError exception.

  26.2.1 The Proxy Constructor

  The Proxy constructor is the %Proxy% intrinsic object and the initial value of
  the Proxy property of the global object. When called as a constructor it
  creates and initializes a new proxy exotic object. Proxy is not intended to be
  called as a function and will throw an exception when called in that manner.
---*/

assert.throws(TypeError, function() {
  class P extends Proxy {}
});
