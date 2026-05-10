// This file was procedurally generated from the following sources:
// - src/class-elements/private-field-as-async-arrow-function.case
// - src/class-elements/default/cls-decl.template
/*---
description: Calling async arrow function returned from private field access (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-private, async-functions, arrow-function, class]
flags: [generated, async]
info: |
    Updated Productions

    CallExpression[Yield, Await]:
      CoverCallExpressionAndAsyncArrowHead[?Yield, ?Await]
      SuperCall[?Yield, ?Await]
      CallExpression[?Yield, ?Await]Arguments[?Yield, ?Await]
      CallExpression[?Yield, ?Await][Expression[+In, ?Yield, ?Await]]
      CallExpression[?Yield, ?Await].IdentifierName
      CallExpression[?Yield, ?Await]TemplateLiteral[?Yield, ?Await]
      CallExpression[?Yield, ?Await].PrivateName

---*/


class C {
  #m = async () => 'test262';

  method() {
    return this.#m();
  }
}

let c = new C();

c.method().then((value) => assert.sameValue(value, 'test262'))
  .then($DONE, $DONE);

