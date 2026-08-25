// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-private-field-super-access.case
// - src/class-elements/syntax/invalid/cls-decl-elements-invalid-syntax.template
/*---
description: Acessing private field from super is not a valid syntax (class declaration)
esid: prod-ClassElement
features: [class-fields-private, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Updated Productions

    MemberExpression[Yield]:
      MemberExpression[?Yield].PrivateName

---*/


$DONOTEVALUATE();

class C {
  #m = function() { return 'test262'; };

  Child = class extends C {
    access() {
      return super.#m;
    }

    method() {
      return super.#m();
    }
  }
}
