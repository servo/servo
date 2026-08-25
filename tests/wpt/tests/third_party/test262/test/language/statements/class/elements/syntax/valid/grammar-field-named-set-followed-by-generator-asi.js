// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: ASI between a field named "set" and a generator method
esid: prod-ClassElement
features: [class-fields-public, class]
info: |
    ClassElement :
      MethodDefinition
      FieldDefinition ;
      static FieldDefinition ;
      ;

    MethodDefinition :
      GeneratorMethod
      set ClassElementName ( PropertySetParameterList ) { FunctionBody }

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
  set
  *a(x) {}
}

class B {
  static set
  *a(x) {}
}

assert(
  A.prototype.hasOwnProperty("a"),
  "(A) The generator is installed on the prototype"
);
assert(
  new A().hasOwnProperty("set"),
  "(A) The field is installed on class instances"
);
assert(
  B.prototype.hasOwnProperty("a"),
  "(B) The generator is installed on the prototype"
);
assert(
  B.hasOwnProperty("set"),
  "(B) The field is installed on class"
);
