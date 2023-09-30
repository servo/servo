/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Validation tests for struct types
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('no_direct_recursion')
  .desc('Test that direct recursion of structures is rejected')
  .params(u => u.combine('target', ['i32', 'S']))
  .fn(t => {
    const wgsl = `
struct S {
  a : ${t.params.target}
}`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });

g.test('no_indirect_recursion')
  .desc('Test that indirect recursion of structures is rejected')
  .params(u => u.combine('target', ['i32', 'S']))
  .fn(t => {
    const wgsl = `
struct S {
  a : T
}
struct T {
  a : ${t.params.target}
}`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });

g.test('no_indirect_recursion_via_array_element')
  .desc('Test that indirect recursion of structures via array element types is rejected')
  .params(u => u.combine('target', ['i32', 'S']))
  .fn(t => {
    const wgsl = `
struct S {
  a : array<${t.params.target}, 4>
}
`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });

g.test('no_indirect_recursion_via_array_size')
  .desc('Test that indirect recursion of structures via array size expressions is rejected')
  .params(u => u.combine('target', ['S1', 'S2']))
  .fn(t => {
    const wgsl = `
struct S1 {
  a : i32,
}
struct S2 {
  a : i32,
  b : array<i32, ${t.params.target}().a + 1>,
}
`;
    t.expectCompileResult(t.params.target === 'S1', wgsl);
  });

g.test('no_indirect_recursion_via_struct_attribute')
  .desc('Test that indirect recursion of structures via struct members is rejected')
  .params(u =>
    u //
      .combine('target', ['S1', 'S2'])
      .combine('attribute', ['align', 'location', 'size'])
  )
  .fn(t => {
    const wgsl = `
struct S1 {
  a : i32
}
struct S2 {
  @${t.params.attribute}(${t.params.target}(4).a) a : i32
}
`;
    t.expectCompileResult(t.params.target === 'S1', wgsl);
  });

g.test('no_indirect_recursion_via_struct_member_nested_in_alias')
  .desc(
    `Test that indirect recursion of structures via struct members is rejected when the member type
    is an alias that contains the structure`
  )
  .params(u => u.combine('target', ['i32', 'A']))
  .fn(t => {
    const wgsl = `
alias A = array<S2, 4>;
struct S1 {
  a : ${t.params.target}
}
struct S2 {
  a : S1
}
`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });
