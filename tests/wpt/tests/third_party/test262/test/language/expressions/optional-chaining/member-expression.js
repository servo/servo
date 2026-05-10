// Copyright 2019 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  optional chain on member expression
info: |
  Left-Hand-Side Expressions
    OptionalExpression:
      MemberExpression OptionalChain
features: [optional-chaining]
---*/

// PrimaryExpression
//   IdentifierReference
const a = {b: 22};
assert.sameValue(22, a?.b);
//   this
function fn () {
  return this?.a
}
assert.sameValue(33, fn.call({a: 33}));
//   Literal
assert.sameValue(undefined, "hello"?.a);
assert.sameValue(undefined, null?.a);
//   ArrayLiteral
assert.sameValue(2, [1, 2]?.[1]);
//   ObjectLiteral
assert.sameValue(44, {a: 44}?.a);
//   FunctionExpression
assert.sameValue('a', (function a () {}?.name));
//   ClassExpression
assert.sameValue('Foo', (class Foo {}?.name));
//   GeneratorFunction
assert.sameValue('a', (function * a () {}?.name));
//   AsyncFunctionExpression
assert.sameValue('a', (async function a () {}?.name));
//   AsyncGeneratorExpression
assert.sameValue('a', (async function * a () {}?.name));
//   RegularExpressionLiteral
assert.sameValue(true, /[a-z]/?.test('a'));
//   TemplateLiteral
assert.sameValue('h', `hello`?.[0]);
//   CoverParenthesizedExpressionAndArrowParameterList
assert.sameValue(undefined, ({a: 33}, null)?.a);
assert.sameValue(33, (undefined, {a: 33})?.a);

//  MemberExpression [ Expression ]
const arr = [{a: 33}];
assert.sameValue(33, arr[0]?.a);
assert.sameValue(undefined, arr[1]?.a);

//  MemberExpression .IdentifierName
const obj = {a: {b: 44}};
assert.sameValue(44, obj.a?.b);
assert.sameValue(undefined, obj.c?.b);

//  MemberExpression TemplateLiteral
function f2 () {
  return {a: 33};
}
function f3 () {}
assert.sameValue(33, f2`hello world`?.a);
assert.sameValue(undefined, f3`hello world`?.a);

//  MemberExpression SuperProperty
class A {
  a () {}
  undf () {
    return super.a?.c;
  }
}
class B extends A {
  dot () {
    return super.a?.name;
  }
  expr () {
    return super['a']?.name;
  }
  undf2 () {
    return super.b?.c;
  }
}
const subcls = new B();
assert.sameValue('a', subcls.dot());
assert.sameValue('a', subcls.expr());
assert.sameValue(undefined, subcls.undf2());
assert.sameValue(undefined, (new A()).undf());

// MemberExpression MetaProperty
class C {
  constructor () {
    assert.sameValue(undefined, new.target?.a);
  }
}
new C();

// new MemberExpression Arguments
class D {
  constructor (val) {
    this.a = val;
  }
}
assert.sameValue(99, new D(99)?.a);
