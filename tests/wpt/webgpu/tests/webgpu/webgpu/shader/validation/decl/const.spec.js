/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Validation tests for const declarations
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('no_direct_recursion')
  .desc('Test that direct recursion of const declarations is rejected')
  .params(u => u.combine('target', ['a', 'b']))
  .fn(t => {
    const wgsl = `
const a : i32 = 42;
const b : i32 = ${t.params.target};
`;
    t.expectCompileResult(t.params.target === 'a', wgsl);
  });

g.test('no_indirect_recursion')
  .desc('Test that indirect recursion of const declarations is rejected')
  .params(u => u.combine('target', ['a', 'b']))
  .fn(t => {
    const wgsl = `
const a : i32 = 42;
const b : i32 = c;
const c : i32 = ${t.params.target};
`;
    t.expectCompileResult(t.params.target === 'a', wgsl);
  });

g.test('no_indirect_recursion_via_array_size')
  .desc('Test that indirect recursion of const declarations via array size expressions is rejected')
  .params(u => u.combine('target', ['a', 'b']))
  .fn(t => {
    const wgsl = `
const a = 4;
const b = c[0];
const c = array<i32, ${t.params.target}>(4, 4, 4, 4);
`;
    t.expectCompileResult(t.params.target === 'a', wgsl);
  });

g.test('no_indirect_recursion_via_struct_attribute')
  .desc('Test that indirect recursion of const declarations via struct members is rejected')
  .params(u =>
    u //
      .combine('target', ['a', 'b'])
      .combine('attribute', ['align', 'location', 'size'])
  )
  .fn(t => {
    const wgsl = `
struct S {
  @${t.params.attribute}(${t.params.target}) a : i32
}
const a = 4;
const b = S(4).a;
`;
    t.expectCompileResult(t.params.target === 'a', wgsl);
  });
