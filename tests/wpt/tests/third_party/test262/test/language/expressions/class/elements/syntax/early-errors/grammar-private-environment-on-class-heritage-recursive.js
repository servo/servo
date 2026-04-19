// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-private-environment-on-class-heritage-recursive.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: It's a SyntaxError if a class expression evaluated on ClassHeritage of a ClassHeritage uses an undeclared private name. (class expression)
esid: prod-ClassElement
features: [class-fields-private, class-fields-public, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Runtime Semantics: ClassDefinitionEvaluation

    ClassTail : ClassHeritage { ClassBody }
        ...
        5. Let outerPrivateEnvironment be the PrivateEnvironment of the running execution context.
        6. Let classPrivateEnvironment be NewDeclarativeEnvironment(outerPrivateEnvironment).
        7. Let classPrivateEnvRec be classPrivateEnvironment's EnvironmentRecord.
        8. If ClassBodyopt is present, then
            a. For each element dn of the PrivateBoundIdentifiers of ClassBodyopt,
              i. Perform classPrivateEnvRec.CreateImmutableBinding(dn, true).
        9. If ClassHeritageopt is not present, then
            a. Let protoParent be the intrinsic object %ObjectPrototype%.
            b. Let constructorParent be the intrinsic object %FunctionPrototype%.
        10. Else,
            a. Set the running execution context's LexicalEnvironment to classScope.
            b. NOTE: The running execution context's PrivateEnvironment is outerPrivateEnvironment when evaluating ClassHeritage.
        ...

---*/


$DONOTEVALUATE();

var C = class extends class extends class { x = this.#foo; } {}
{
  #foo;
};
