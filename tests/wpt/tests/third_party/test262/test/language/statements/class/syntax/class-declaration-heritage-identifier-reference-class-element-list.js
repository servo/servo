// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    ClassHeritage[Yield] :
      extends LeftHandSideExpression[?Yield]

    LeftHandSideExpression :
      NewExpression
      ...

    NewExpression :
      MemberExpression
      ...

    MemberExpression :
      PrimaryExpression
      ...

    PrimaryExpression :
      IdentifierReference
      ...

    ClassDeclaration:
      class BindingIdentifier ClassTail

    ClassTail:
      ... { ClassBodyopt }

    ClassBody[Yield] :
      ClassElementList[?Yield]


    ClassElementList[Yield] :
      ClassElement[?Yield]
      ClassElementList[?Yield] ClassElement[?Yield]

    ClassElement[Yield] :
      MethodDefinition[?Yield]
      static MethodDefinition[?Yield]
      ;

---*/
class A {}
class B extends A {
  method() {}
  static method() {}
  ;
}

assert.sameValue(typeof B, "function");
assert.sameValue(typeof B.prototype.method, "function");
assert.sameValue(typeof B.method, "function");
