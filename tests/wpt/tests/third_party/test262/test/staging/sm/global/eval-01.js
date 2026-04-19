/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
var a = 9;
var global = this;

function test() {
    var a = 0;

    // direct eval sees local a
    assert.sameValue(eval('a+1'), 1);
    assert.sameValue(eval('eval("a+1")'), 1);

    // indirect: using a name other than 'eval'
    var foo = eval;
    assert.sameValue(foo('a+1'), 10);
    assert.sameValue(eval('foo("a+1")'), 10); // outer eval is direct, inner foo("a+1") is indirect

    // indirect: qualified method call
    assert.sameValue(this.eval("a+1"), 10);
    assert.sameValue(global.eval("a+1"), 10);
    var obj = {foo: eval, eval: eval};
    assert.sameValue(obj.foo('a+1'), 10);
    assert.sameValue(obj.eval('a+1'), 10);
    var name = "eval";
    assert.sameValue(obj[name]('a+1'), 10);
    assert.sameValue([eval][0]('a+1'), 10);

    // indirect: not called from a CallExpression at all
    assert.sameValue(eval.call(undefined, 'a+1'), 10);
    assert.sameValue(eval.call(global, 'a+1'), 10);
    assert.sameValue(eval.apply(undefined, ['a+1']), 10);
    assert.sameValue(eval.apply(global, ['a+1']), 10);
    assert.sameValue(['a+1'].map(eval)[0], 10);
}

test();
