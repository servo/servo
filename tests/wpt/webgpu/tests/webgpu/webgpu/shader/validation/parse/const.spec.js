/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for @const`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('placement').
desc('Tests @const is not allowed to appear').
params((u) =>
u.combine('scope', [
'private-var',
'storage-var',
'struct-member',
'fn-decl',
'fn-param',
'fn-var',
'fn-return',
'while-stmt',
undefined]
)
).
fn((t) => {
  const scope = t.params.scope;

  const attr = '@const';
  const code = `
      ${scope === 'private-var' ? attr : ''}
      var<private> priv_var : i32;

      ${scope === 'storage-var' ? attr : ''}
      @group(0) @binding(0)
      var<storage> stor_var : i32;

      struct A {
        ${scope === 'struct-member' ? attr : ''}
        a : i32,
      }

      @vertex
      ${scope === 'fn-decl' ? attr : ''}
      fn f(
        ${scope === 'fn-param' ? attr : ''}
        @location(0) b : i32,
      ) -> ${scope === 'fn-return' ? attr : ''} @builtin(position) vec4f {
        ${scope === 'fn-var' ? attr : ''}
        var<function> func_v : i32;

        ${scope === 'while-stmt' ? attr : ''}
        while false {}

        return vec4(1, 1, 1, 1);
      }
    `;

  t.expectCompileResult(scope === undefined, code);
});