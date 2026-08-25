// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%throwtypeerror%
description: >
  %ThrowTypeError% does not have an own "caller" property.
info: |
  %ThrowTypeError% ( )

  The %ThrowTypeError% intrinsic is an anonymous built-in function
  object that is defined once for each realm.

  16.2 Forbidden Extensions

    Other than as defined in this specification, ECMAScript Function
    objects defined using syntactic constructors in strict mode code
    must not be created with own properties named "caller" or
    "arguments" other than those that are created by applying the
    AddRestrictedFunctionProperties abstract operation (9.2.7) to
    the function. [...] Built-in functions, strict mode functions
    created using the Function constructor, generator functions
    created using the Generator constructor, and functions created
    using the bind method also must not be created with such own
    properties.
---*/

var ThrowTypeError = Object.getOwnPropertyDescriptor(function() {
  "use strict";
  return arguments;
}(), "callee").get;

assert.sameValue(Object.prototype.hasOwnProperty.call(ThrowTypeError, "caller"), false);
