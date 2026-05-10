// This file was procedurally generated from the following sources:
// - src/class-elements/private-static-method-usage-inside-nested-class.case
// - src/class-elements/default/cls-decl.template
/*---
description: PrivateName of private static method is available inside inner classes (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-static-methods-private, class-static-fields-public, class]
flags: [generated]
info: |
    Updated Productions

    CallExpression[Yield, Await]:
      CoverCallExpressionAndAsyncArrowHead[?Yield, ?Await]
      SuperCall[?Yield, ?Await]
      CallExpression[?Yield, ?Await]Arguments[?Yield, ?Await]
      CallExpression[?Yield, ?Await][Expression[+In, ?Yield, ?Await]]
      CallExpression[?Yield, ?Await].IdentifierName
      CallExpression[?Yield, ?Await]TemplateLiteral[?Yield, ?Await]
      CallExpression[?Yield, ?Await].PrivateIdentifier

    ClassTail : ClassHeritage { ClassBody }
      ...
      6. Let classPrivateEnvironment be NewDeclarativeEnvironment(outerPrivateEnvironment).
      7. Let classPrivateEnvRec be classPrivateEnvironment's EnvironmentRecord.
      ...
      15. Set the running execution context's LexicalEnvironment to classScope.
      16. Set the running execution context's PrivateEnvironment to classPrivateEnvironment.
      ...
      33. If PrivateBoundIdentifiers of ClassBody contains a Private Name P such that P's [[Kind]] field is either "method" or "accessor" and P's [[Brand]] is F,
        a. PrivateBrandAdd(F, F).
      34. For each item fieldRecord in order from staticFields,
        a. Perform ? DefineField(F, field).

      FieldDefinition : ClassElementName Initializer_opt
        1. Let name be the result of evaluating ClassElementName.
        2. ReturnIfAbrupt(name).
        3. If Initializer_opt is present,
          a. Let lex be the Lexical Environment of the running execution context.
          b. Let formalParameterList be an instance of the production FormalParameters : [empty].
          c. Let privateScope be the PrivateEnvironment of the running execution context.
          d. Let initializer be FunctionCreate(Method, formalParameterList, Initializer, lex, true, privateScope).
          e. Perform MakeMethod(initializer, homeObject).
          f. Let isAnonymousFunctionDefinition be IsAnonymousFunctionDefinition(Initializer).
        4. Else,
          a. Let initializer be empty.
          b. Let isAnonymousFunctionDeclaration be false.
        5. Return a Record { [[Name]]: name, [[Initializer]]: initializer, [[IsAnonymousFunctionDefinition]]: isAnonymousFunctionDefinition }.

    MemberExpression : MemberExpression.PrivateIdentifier
      1. Let baseReference be the result of evaluating MemberExpression.
      2. Let baseValue be ? GetValue(baseReference).
      3. Let bv be ? RequireObjectCoercible(baseValue).
      4. Let fieldNameString be the StringValue of PrivateIdentifier.
      5. Return MakePrivateReference(bv, fieldNameString).

    MakePrivateReference(baseValue, privateIdentifier)
      1. Let env be the running execution context's PrivateEnvironment.
      2. Let privateNameBinding be ? ResolveBinding(privateIdentifier, env).
      3. Let privateName be GetValue(privateNameBinding).
      4. Assert: privateName is a Private Name.
      5. Return a value of type Reference whose base value is baseValue, whose referenced name is privateName, whose strict reference flag is true.

---*/


class C {
  static #m() {
    return 'outer class';
  }

  static B = class {
    static methodAccess(o) {
      return o.#m();
    }
  }
}

assert.sameValue(C.B.methodAccess(C), 'outer class');
assert.throws(TypeError, function() {
  C.B.methodAccess(C.B);
}, 'accessed static private method from an arbritary object');
