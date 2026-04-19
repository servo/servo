// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
var BUGNUMBER = 1566143;
var summary = "Implement the Optional Chain operator (?.) proposal";

// These tests are originally from webkit.
// webkit specifics have been removed and error messages have been updated.

function testBasicSuccessCases() {
    assert.sameValue(undefined?.valueOf(), undefined);
    assert.sameValue(null?.valueOf(), undefined);
    assert.sameValue(true?.valueOf(), true);
    assert.sameValue(false?.valueOf(), false);
    assert.sameValue(0?.valueOf(), 0);
    assert.sameValue(1?.valueOf(), 1);
    assert.sameValue(''?.valueOf(), '');
    assert.sameValue('hi'?.valueOf(), 'hi');
    assert.sameValue(({})?.constructor, Object);
    assert.sameValue(({ x: 'hi' })?.x, 'hi');
    assert.sameValue([]?.length, 0);
    assert.sameValue(['hi']?.length, 1);

    assert.sameValue(undefined?.['valueOf'](), undefined);
    assert.sameValue(null?.['valueOf'](), undefined);
    assert.sameValue(true?.['valueOf'](), true);
    assert.sameValue(false?.['valueOf'](), false);
    assert.sameValue(0?.['valueOf'](), 0);
    assert.sameValue(1?.['valueOf'](), 1);
    assert.sameValue(''?.['valueOf'](), '');
    assert.sameValue('hi'?.['valueOf'](), 'hi');
    assert.sameValue(({})?.['constructor'], Object);
    assert.sameValue(({ x: 'hi' })?.['x'], 'hi');
    assert.sameValue([]?.['length'], 0);
    assert.sameValue(['hi']?.[0], 'hi');

    assert.sameValue(undefined?.(), undefined);
    assert.sameValue(null?.(), undefined);
    assert.sameValue((() => 3)?.(), 3);
}

function testBasicFailureCases() {
    assert.throws(TypeError, () => true?.(), 'true is not a function');
    assert.throws(TypeError, () => false?.(), 'false is not a function');
    assert.throws(TypeError, () => 0?.(), '0 is not a function');
    assert.throws(TypeError, () => 1?.(), '1 is not a function');
    assert.throws(TypeError, () => ''?.(), '"" is not a function');
    assert.throws(TypeError, () => 'hi'?.(), '"hi" is not a function');
    assert.throws(TypeError, () => ({})?.(), '({}) is not a function');
    assert.throws(TypeError, () => ({ x: 'hi' })?.(), '({x:"hi"}) is not a function');
    assert.throws(TypeError, () => []?.(), '[] is not a function');
    assert.throws(TypeError, () => ['hi']?.(), '[...] is not a function');
}

testBasicSuccessCases();

testBasicFailureCases();

assert.throws(TypeError, () => ({})?.i(), '(intermediate value).i is not a function');
assert.sameValue(({}).i?.(), undefined);
assert.sameValue(({})?.i?.(), undefined);
assert.throws(TypeError, () => ({})?.['i'](), '(intermediate value)["i"] is not a function');
assert.sameValue(({})['i']?.(), undefined);
assert.sameValue(({})?.['i']?.(), undefined);

assert.throws(TypeError, () => ({})?.a['b'], '(intermediate value).a is undefined');
assert.sameValue(({})?.a?.['b'], undefined);
assert.sameValue(null?.a['b']().c, undefined);
assert.throws(TypeError, () => ({})?.['a'].b, '(intermediate value)["a"] is undefined');
assert.sameValue(({})?.['a']?.b, undefined);
assert.sameValue(null?.['a'].b()['c'], undefined);
assert.sameValue(null?.()().a['b'], undefined);

const o0 = { a: { b() { return this._b.bind(this); }, _b() { return this.__b; }, __b: { c: 42 } } };
assert.sameValue(o0?.a?.['b']?.()?.()?.c, 42);
assert.sameValue(o0?.i?.['j']?.()?.()?.k, undefined);
assert.sameValue((o0.a?._b)?.().c, 42);
assert.sameValue((o0.a?._b)().c, 42);
assert.sameValue((o0.a?.b?.())?.().c, 42);
assert.sameValue((o0.a?.['b']?.())?.().c, 42);

assert.sameValue(({ undefined: 3 })?.[null?.a], 3);
assert.sameValue((() => 3)?.(null?.a), 3);

const o1 = { count: 0, get x() { this.count++; return () => {}; } };
o1.x?.y;
assert.sameValue(o1.count, 1);
o1.x?.['y'];
assert.sameValue(o1.count, 2);
o1.x?.();
assert.sameValue(o1.count, 3);
null?.(o1.x);
assert.sameValue(o1.count, 3);

assert.sameValue(delete undefined?.foo, true);
assert.sameValue(delete null?.foo, true);
assert.sameValue(delete undefined?.['foo'], true);
assert.sameValue(delete null?.['foo'], true);
assert.sameValue(delete undefined?.(), true);
assert.sameValue(delete null?.(), true);
assert.sameValue(delete ({}).a?.b?.b, true);
assert.sameValue(delete ({a : {b: undefined}}).a?.b?.b, true);
assert.sameValue(delete ({a : {b: undefined}}).a?.["b"]?.["b"], true);

const o2 = { x: 0, y: 0, z() {} };
assert.sameValue(delete o2?.x, true);
assert.sameValue(o2.x, undefined);
assert.sameValue(o2.y, 0);
assert.sameValue(delete o2?.x, true);
assert.sameValue(delete o2?.['y'], true);
assert.sameValue(o2.y, undefined);
assert.sameValue(delete o2?.['y'], true);
assert.sameValue(delete o2.z?.(), true);

function greet(name) { return `hey, ${name}${this.suffix ?? '.'}`; }
assert.sameValue(eval?.('greet("world")'), 'hey, world.');
assert.sameValue(greet?.call({ suffix: '!' }, 'world'), 'hey, world!');
assert.sameValue(greet.call?.({ suffix: '!' }, 'world'), 'hey, world!');
assert.sameValue(null?.call({ suffix: '!' }, 'world'), undefined);
assert.sameValue(({}).call?.({ suffix: '!' }, 'world'), undefined);
assert.sameValue(greet?.apply({ suffix: '?' }, ['world']), 'hey, world?');
assert.sameValue(greet.apply?.({ suffix: '?' }, ['world']), 'hey, world?');
assert.sameValue(null?.apply({ suffix: '?' }, ['world']), undefined);
assert.sameValue(({}).apply?.({ suffix: '?' }, ['world']), undefined);
assert.throws(SyntaxError, () => eval('class C {} class D extends C { foo() { return super?.bar; } }'));
assert.throws(SyntaxError, () => eval('class C {} class D extends C { foo() { return super?.["bar"]; } }'));
assert.throws(SyntaxError, () => eval('class C {} class D extends C { constructor() { super?.(); } }'));
assert.throws(SyntaxError, () => eval('const o = { C: class {} }; new o?.C();'));
assert.throws(SyntaxError, () => eval('const o = { C: class {} }; new o?.["C"]();'));
assert.throws(SyntaxError, () => eval('class C {} new C?.();'));
assert.throws(SyntaxError, () => eval('function foo() { new?.target; }'));
assert.throws(SyntaxError, () => eval('function tag() {} tag?.``;'));
assert.throws(SyntaxError, () => eval('const o = { tag() {} }; o?.tag``;'));
assert.throws(ReferenceError, () => eval('`${G}`?.r'));

// NOT an optional chain
assert.sameValue(false?.4:5, 5);

// Special case: binary operators that follow a binary expression
assert.throws(ReferenceError, () => eval('(0 || 1 << x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 >> x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 >>> x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 + x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 - x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 % x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 / x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 == x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 != x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 !== x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 === x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 <= x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 >= x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 ** x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 | x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 & x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || 1 instanceof x)?.$'));
assert.throws(ReferenceError, () => eval('(0 || "foo" in x)?.$'));

function testSideEffectCountFunction() {
  let count = 0;
  let a = {
    b: {
      c: {
        d: () => {
          count++;
          return a;
        }
      }
    }
  }

  a.b.c.d?.()?.b?.c?.d

  assert.sameValue(count, 1);
}

function testSideEffectCountGetters() {
  let count = 0;
  let a = {
    get b() {
      count++;
      return { c: {} };
    }
  }

  a.b?.c?.d;
  assert.sameValue(count, 1);
  a.b?.c?.d;
  assert.sameValue(count, 2);
}

testSideEffectCountFunction();
testSideEffectCountGetters();

// stress test SM
assert.sameValue(({a : {b: undefined}}).a.b?.()()(), undefined);
assert.sameValue(({a : {b: undefined}}).a.b?.()?.()(), undefined);
assert.sameValue(({a : {b: () => undefined}}).a.b?.()?.(), undefined);
assert.throws(TypeError, () => delete ({a : {b: undefined}}).a?.b.b.c, '(intermediate value).a.b is undefined');
assert.sameValue(delete ({a : {b: undefined}}).a?.["b"]?.["b"], true);
assert.sameValue(delete undefined ?.x[y+1], true);
assert.throws(TypeError, () => (({a : {b: () => undefined}}).a.b?.())(), 'undefined is not a function');
assert.throws(TypeError, () => (delete[1]?.r[delete[1]?.r1]), "[...].r is undefined");
assert.throws(TypeError, () => (delete[1]?.r[[1]?.r1]), "[...].r is undefined");
