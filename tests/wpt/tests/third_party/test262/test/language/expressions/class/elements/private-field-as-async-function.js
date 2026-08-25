// This file was procedurally generated from the following sources:
// - src/class-elements/private-field-as-async-function.case
// - src/class-elements/default/cls-expr.template
/*---
description: Calling async function returned from private field access (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-private, async-functions, class]
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


var C = class {
  #m = async function() { return 'test262'; };

  method() {
    return this.#m();
  }
}

let c = new C();

c.method().then((value) => assert.sameValue(value, 'test262'))
  .then($DONE, $DONE);

