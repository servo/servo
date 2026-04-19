// Copyright (C) 2021 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Evaluation order when resolving private fields.
esid: sec-runtime-semantics-keyeddestructuringassignmentevaluation
info: |
  13.15.5.6 Runtime Semantics: KeyedDestructuringAssignmentEvaluation
    1. If DestructuringAssignmentTarget is neither an ObjectLiteral nor an ArrayLiteral, then
      a. Let lref be the result of evaluating DestructuringAssignmentTarget.
      b. ReturnIfAbrupt(lref).
  2. Let v be ? GetV(value, propertyName).
  3. ...

features: [class, class-fields-private]
---*/

class Base {
  constructor(o) {
    return o;
  }
}

class C extends Base {
  #field;

  m() {
    var init = () => new C(this);

    var object = {
      get a() {
        init();

        return "pass";
      }
    };

    ({a: this.#field} = object);

    assert.sameValue(this.#field, "pass");
  }
}

C.prototype.m.call({});
