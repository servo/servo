/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/






/** A selection of different types used by statement validation tests */
export const kTestTypes = {
  bool: { value: 'true' },
  i32: { value: '1i' },
  u32: { value: '1u' },
  f32: { value: '1f' },
  f16: { value: '1h', requires: 'f16' },
  'abstract-int': { value: '1' },
  'abstract-float': { value: '1.0' },
  vec2af: { value: 'vec2(1.0)' },
  vec3af: { value: 'vec3(1.0)' },
  vec4af: { value: 'vec4(1.0)' },
  vec2ai: { value: 'vec2(1)' },
  vec3ai: { value: 'vec3(1)' },
  vec4ai: { value: 'vec4(1)' },
  vec2f: { value: 'vec2f(1)' },
  vec3h: { value: 'vec3h(1)', requires: 'f16' },
  vec4u: { value: 'vec4u(1)' },
  vec3b: { value: 'vec3<bool>(true)' },
  mat2x3f: { value: 'mat2x3f(1, 2, 3, 4, 5, 6)' },
  mat4x2h: { value: 'mat4x2h(1, 2, 3, 4, 5, 6, 7, 8)', requires: 'f16' },
  array: { value: 'array<i32, 4>(1, 2, 3, 4)' },
  atomic: { value: 'A', header: 'var<workgroup> A : atomic<i32>;' },
  struct: { value: 'Str(1)', header: 'struct Str{ i : i32 }' },
  texture: { value: 'T', header: '@group(0) @binding(0) var T : texture_2d<f32>;' },
  sampler: { value: 'S', header: '@group(0) @binding(1) var S : sampler;' }
};