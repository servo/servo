// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-all-private-names-valid
description: Referencing privatename in class within class does not error.
info: |
  Static Semantics: AllPrivateNamesValid

  AllPrivateNamesValid is an abstract operation which takes names as an argument.

    MemberExpression : MemberExpression . PrivateName
      1. If StringValue of PrivateName is in names, return true.
      2. Return false.

    CallExpression : CallExpression . PrivateName
      1. If StringValue of PrivateName is in names, return true.
      2. Return false.

    ClassBody : ClassElementList
      1. Let newNames be the concatenation of names with PrivateBoundNames of ClassBody.
      2. Return AllPrivateNamesValid of ClassElementList with the argument newNames.

    For all other grammatical productions, recurse on subexpressions/substatements, passing in the names of the caller. If all pieces return true, then return true. If any returns false, return false.

features: [class, class-fields-private]
---*/

class outer {
  #x = 42;

  f() {
    var self = this;
    return class inner {
      g() {
        return self.#x;
      }
    }
  }
}

var innerclass = new outer().f();
var test = new innerclass().g();

assert.sameValue(test, 42);
