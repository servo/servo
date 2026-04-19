// This file was procedurally generated from the following sources:
// - src/class-elements/init-err-evaluation.case
// - src/class-elements/default/cls-decl.template
/*---
description: Return abrupt completion evaluating the field initializer (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class]
flags: [generated]
info: |
    [[Construct]] ( argumentsList, newTarget)

    8. If kind is "base", then
      a. Perform OrdinaryCallBindThis(F, calleeContext, thisArgument).
      b. Let result be InitializeInstanceFields(thisArgument, F).
      ...
    ...
    11. Let result be OrdinaryCallEvaluateBody(F, argumentsList).
    ...

---*/
var x = 0;
function fn1() { x += 1; }
function fn2() { throw new Test262Error(); }


class C {
  x = fn1();
  y = fn2();
  z = fn1();
}

assert.throws(Test262Error, function() {
  new C();
});

assert.sameValue(x, 1);
