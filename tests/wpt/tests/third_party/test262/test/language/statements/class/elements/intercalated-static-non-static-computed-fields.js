// This file was procedurally generated from the following sources:
// - src/class-elements/intercalated-static-non-static-computed-fields.case
// - src/class-elements/default/cls-decl.template
/*---
description: Computed class fields are executed in the order they are delcared, regardless it is static or instance field (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-static-fields-public, class-fields-public, class]
flags: [generated]
includes: [propertyHelper.js]
info: |
    ClassTail : ClassHeritage { ClassBody }
      ...
      28. For each ClassElement e in order from elements,
        a. If IsStatic of e is false, then
          i. Let field be the result of performing ClassElementEvaluation for e with arguments proto and false.
        b. Else,
          i. Let field be the result of performing PropertyDefinitionEvaluation for mClassElementEvaluation for e with arguments F and false.
        c. If field is an abrupt completion, then
          ...
        d. If field is not empty,
          i. If IsStatic of e is false, append field to instanceFields.
          ii. Otherwise, append field to staticFields.
       ...
       34. For each item fieldRecord in order from staticFields,
         a. Perform ? DefineField(F, field).
       ...

    [[Construct]] (argumentsList, newTarget)
      ...
      8. If kind is "base", then
        a. Perform OrdinaryCallBindThis(F, calleeContext, thisArgument).
        b. Let result be InitializeInstanceFields(thisArgument, F).
        c. If result is an abrupt completion, then
          i. Remove calleeContext from execution context stack and restore callerContext as the running execution context.
          ii. Return Completion(result).

---*/

let i = 0;


class C {
  [i++] = i++;
  static [i++] = i++;
  [i++] = i++;
}

let c = new C();

// It is important to notice that static field initializer will run before any instance initializer
verifyProperty(c, "0", {
  value: 4,
  enumerable: true,
  writable: true,
  configurable: true
});

verifyProperty(c, "2", {
  value: 5,
  enumerable: true,
  writable: true,
  configurable: true
});

verifyProperty(C, "1", {
  value: 3,
  enumerable: true,
  writable: true,
  configurable: true
});

assert.sameValue(i, 6);
assert.sameValue(c.hasOwnProperty('1'), false);
assert.sameValue(C.hasOwnProperty('0'), false);
assert.sameValue(C.hasOwnProperty('2'), false);

