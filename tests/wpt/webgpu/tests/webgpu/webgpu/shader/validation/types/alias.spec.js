/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Validation tests for type aliases
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('no_direct_recursion')
  .desc('Test that direct recursion of type aliases is rejected')
  .params(u => u.combine('target', ['i32', 'T']))
  .fn(t => {
    const wgsl = `alias T = ${t.params.target};`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });

g.test('no_indirect_recursion')
  .desc('Test that indirect recursion of type aliases is rejected')
  .params(u => u.combine('target', ['i32', 'S']))
  .fn(t => {
    const wgsl = `
alias S = T;
alias T = ${t.params.target};
`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });

g.test('no_indirect_recursion_via_vector_element')
  .desc('Test that indirect recursion of type aliases via vector element types is rejected')
  .params(u => u.combine('target', ['i32', 'V']))
  .fn(t => {
    const wgsl = `
alias V = vec4<T>;
alias T = ${t.params.target};
`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });

g.test('no_indirect_recursion_via_matrix_element')
  .desc('Test that indirect recursion of type aliases via matrix element types is rejected')
  .params(u => u.combine('target', ['f32', 'M']))
  .fn(t => {
    const wgsl = `
alias M = mat4x4<T>;
alias T = ${t.params.target};
`;
    t.expectCompileResult(t.params.target === 'f32', wgsl);
  });

g.test('no_indirect_recursion_via_array_element')
  .desc('Test that indirect recursion of type aliases via array element types is rejected')
  .params(u => u.combine('target', ['i32', 'A']))
  .fn(t => {
    const wgsl = `
alias A = array<T, 4>;
alias T = ${t.params.target};
`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });

g.test('no_indirect_recursion_via_array_size')
  .desc('Test that indirect recursion of type aliases via array size expressions is rejected')
  .params(u => u.combine('target', ['i32', 'A']))
  .fn(t => {
    const wgsl = `
alias A = array<i32, T(1)>;
alias T = ${t.params.target};
`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });

g.test('no_indirect_recursion_via_atomic')
  .desc('Test that indirect recursion of type aliases via atomic types is rejected')
  .params(u => u.combine('target', ['i32', 'A']))
  .fn(t => {
    const wgsl = `
alias A = atomic<T>;
alias T = ${t.params.target};
`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });

g.test('no_indirect_recursion_via_ptr_store_type')
  .desc('Test that indirect recursion of type aliases via pointer store types is rejected')
  .params(u => u.combine('target', ['i32', 'P']))
  .fn(t => {
    const wgsl = `
alias P = ptr<function, T>;
alias T = ${t.params.target};
`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });

g.test('no_indirect_recursion_via_struct_member')
  .desc('Test that indirect recursion of type aliases via struct members is rejected')
  .params(u => u.combine('target', ['i32', 'S']))
  .fn(t => {
    const wgsl = `
struct S {
  a : T
}
alias T = ${t.params.target};
`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });

g.test('no_indirect_recursion_via_struct_attribute')
  .desc('Test that indirect recursion of type aliases via struct members is rejected')
  .params(u =>
    u //
      .combine('target', ['i32', 'S'])
      .combine('attribute', ['align', 'location', 'size'])
  )
  .fn(t => {
    const wgsl = `
struct S {
  @${t.params.attribute}(T(4)) a : f32
}
alias T = ${t.params.target};
`;
    t.expectCompileResult(t.params.target === 'i32', wgsl);
  });
