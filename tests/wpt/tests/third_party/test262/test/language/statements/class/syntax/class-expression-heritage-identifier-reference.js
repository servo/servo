// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    ClassExpression[Yield,GeneratorParameter] :
      class BindingIdentifier[?Yield]opt ClassTail[?Yield,?GeneratorParameter]

    ClassTail[Yield,GeneratorParameter] :
      [~GeneratorParameter] ClassHeritage[?Yield]opt { ClassBody[?Yield]opt }
      [+GeneratorParameter] ClassHeritageopt { ClassBodyopt }
---*/
class A {}
var B = class extends A {}

assert.sameValue(typeof B, "function");
