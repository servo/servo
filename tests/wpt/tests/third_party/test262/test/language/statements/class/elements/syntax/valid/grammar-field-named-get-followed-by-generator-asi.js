// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: ASI between a field named "get" and a generator method
esid: prod-ClassElement
features: [class-fields-public, class-static-fields-public, class, generators]
info: |
    ClassElement :
      MethodDefinition
      FieldDefinition ;
      static FieldDefinition ;
      ;

    MethodDefinition :
      GeneratorMethod
      get ClassElementName () { FunctionBody }

    GeneratorMethod :
       * ClassElementName ( UniqueFormalParameters ) { GeneratorBody }

    FieldDefinition :
      ClassElementName Initializer _opt

    ClassElementName :
      PropertyName
      PrivateName

    PropertyName :
      LiteralPropertyName
      ComputedPropertyName

    LiteralPropertyName :
      IdentifierName
      StringLiteral
      NumericLiteral
---*/

class A {
  get
  *a() {}
}

class B {
  static get
  *a() {}
}

assert(
  A.prototype.hasOwnProperty("a"),
  "(A) The generator is installed on the prototype"
);
assert(
  new A().hasOwnProperty("get"),
  "(A) The field is installed on class instances"
);
assert(
  B.prototype.hasOwnProperty("a"),
  "(B) The generator is installed on the prototype"
);
assert(
  B.hasOwnProperty("get"),
  "(B) The field is installed on class"
);
