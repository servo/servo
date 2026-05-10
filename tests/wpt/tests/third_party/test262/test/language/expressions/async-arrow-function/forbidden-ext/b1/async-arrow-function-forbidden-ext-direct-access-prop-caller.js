// This file was procedurally generated from the following sources:
// - src/function-forms/forbidden-ext-direct-access-prop-caller.case
// - src/function-forms/forbidden-extensions/bullet-one/async-arrow-function.template
/*---
description: Forbidden extension, o.caller (async arrow function expression)
esid: sec-async-arrow-function-definitions
features: [arrow-function, async-functions]
flags: [generated, noStrict, async]
info: |
    AsyncArrowFunction :
      ...
      CoverCallExpressionAndAsyncArrowHead => AsyncConciseBody

    AsyncConciseBody :
      { AsyncFunctionBody }


    ECMAScript function objects defined using syntactic constructors in strict mode code must
    not be created with own properties named "caller" or "arguments". Such own properties also
    must not be created for function objects defined using an ArrowFunction, MethodDefinition,
    GeneratorDeclaration, GeneratorExpression, AsyncGeneratorDeclaration, AsyncGeneratorExpression,
    ClassDeclaration, ClassExpression, AsyncFunctionDeclaration, AsyncFunctionExpression, or
    AsyncArrowFunction regardless of whether the definition is contained in strict mode code.
    Built-in functions, strict functions created using the Function constructor, generator functions
    created using the Generator constructor, async functions created using the AsyncFunction
    constructor, and functions created using the bind method also must not be created with such own
    properties.

---*/


var callCount = 0;
var f;
f = async () => {
  assert.sameValue(f.hasOwnProperty("caller"), false);
  callCount++;
};

f()
  .then(() => {
    assert.sameValue(callCount, 1, 'function body evaluated');
  }, $DONE).then($DONE, $DONE);
