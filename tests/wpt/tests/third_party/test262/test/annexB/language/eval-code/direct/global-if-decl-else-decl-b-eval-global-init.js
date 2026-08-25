// This file was procedurally generated from the following sources:
// - src/annex-b-fns/eval-global-init.case
// - src/annex-b-fns/eval-global/direct-if-decl-else-decl-b.template
/*---
description: Variable binding is initialized to `undefined` in outer scope (IfStatement with a declaration in both statement positions in eval code)
esid: sec-functiondeclarations-in-ifstatement-statement-clauses
flags: [generated, noStrict]
includes: [fnGlobalObject.js, propertyHelper.js]
info: |
    The following rules for IfStatement augment those in 13.6:

    IfStatement[Yield, Return]:
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else Statement[?Yield, ?Return]
        if ( Expression[In, ?Yield] ) Statement[?Yield, ?Return] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield] else FunctionDeclaration[?Yield]
        if ( Expression[In, ?Yield] ) FunctionDeclaration[?Yield]


    B.3.3.3 Changes to EvalDeclarationInstantiation

    [...]
    i. If varEnvRec is a global Environment Record, then
       i. Perform ? varEnvRec.CreateGlobalFunctionBinding(F, undefined, true).
    [...]

---*/

eval(
  'var global = fnGlobalObject();\
  assert.sameValue(f, undefined, "binding is initialized to `undefined`");\
  \
  verifyProperty(global, "f", {\
    enumerable: true,\
    writable: true,\
    configurable: true\
  });if (false) function _f() {} else function f() {  }'
);
