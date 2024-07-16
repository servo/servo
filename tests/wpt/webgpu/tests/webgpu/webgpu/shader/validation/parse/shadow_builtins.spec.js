/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for identifiers`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('function_param').
desc(
  `Test that a function param can shadow a builtin, but the builtin is available for other params.`
).
fn((t) => {
  const code = `
fn f(f: i32, i32: i32, t: i32) -> i32 { return i32; }
    `;
  t.expectCompileResult(true, code);
});

const kTests = {
  abs: {
    keyword: `abs`,
    src: `_ = abs(1);`
  },
  acos: {
    keyword: `acos`,
    src: `_ = acos(.2);`
  },
  acosh: {
    keyword: `acosh`,
    src: `_ = acosh(1.2);`
  },
  all: {
    keyword: `all`,
    src: `_ = all(true);`
  },
  any: {
    keyword: `any`,
    src: `_ = any(true);`
  },
  array_templated: {
    keyword: `array`,
    src: `_ = array<i32, 2>(1, 2);`
  },
  array: {
    keyword: `array`,
    src: `_ = array(1, 2);`
  },
  array_length: {
    keyword: `arrayLength`,
    src: `_ = arrayLength(&placeholder.rt_arr);`
  },
  asin: {
    keyword: `asin`,
    src: `_ = asin(.2);`
  },
  asinh: {
    keyword: `asinh`,
    src: `_ = asinh(1.2);`
  },
  atan: {
    keyword: `atan`,
    src: `_ = atan(1.2);`
  },
  atanh: {
    keyword: `atanh`,
    src: `_ = atanh(.2);`
  },
  atan2: {
    keyword: `atan2`,
    src: `_ = atan2(1.2, 2.3);`
  },
  bool: {
    keyword: `bool`,
    src: `_ = bool(1);`
  },
  bitcast: {
    keyword: `bitcast`,
    src: `_ = bitcast<f32>(1i);`
  },
  ceil: {
    keyword: `ceil`,
    src: `_ = ceil(1.23);`
  },
  clamp: {
    keyword: `clamp`,
    src: `_ = clamp(1, 2, 3);`
  },
  cos: {
    keyword: `cos`,
    src: `_ = cos(2);`
  },
  cosh: {
    keyword: `cosh`,
    src: `_ = cosh(2.2);`
  },
  countLeadingZeros: {
    keyword: `countLeadingZeros`,
    src: `_ = countLeadingZeros(1);`
  },
  countOneBits: {
    keyword: `countOneBits`,
    src: `_ = countOneBits(1);`
  },
  countTrailingZeros: {
    keyword: `countTrailingZeros`,
    src: `_ = countTrailingZeros(1);`
  },
  cross: {
    keyword: `cross`,
    src: `_ = cross(vec3(1, 2, 3), vec3(4, 5, 6));`
  },
  degrees: {
    keyword: `degrees`,
    src: `_ = degrees(1);`
  },
  determinant: {
    keyword: `determinant`,
    src: `_ = determinant(mat2x2(1, 2, 3, 4));`
  },
  distance: {
    keyword: `distance`,
    src: `_ = distance(1, 2);`
  },
  dot: {
    keyword: `dot`,
    src: `_ = dot(vec2(1, 2,), vec2(2, 3));`
  },
  dot4U8Packed: {
    keyword: `dot4U8Packed`,
    src: `_ = dot4U8Packed(1, 2);`
  },
  dot4I8Packed: {
    keyword: `dot4I8Packed`,
    src: `_ = dot4I8Packed(1, 2);`
  },
  dpdx: {
    keyword: `dpdx`,
    src: `_ = dpdx(2);`
  },
  dpdxCoarse: {
    keyword: `dpdxCoarse`,
    src: `_ = dpdxCoarse(2);`
  },
  dpdxFine: {
    keyword: `dpdxFine`,
    src: `_ = dpdxFine(2);`
  },
  dpdy: {
    keyword: `dpdy`,
    src: `_ = dpdy(2);`
  },
  dpdyCoarse: {
    keyword: `dpdyCoarse`,
    src: `_ = dpdyCoarse(2);`
  },
  dpdyFine: {
    keyword: `dpdyFine`,
    src: `_ = dpdyFine(2);`
  },
  exp: {
    keyword: `exp`,
    src: `_ = exp(1);`
  },
  exp2: {
    keyword: `exp2`,
    src: `_ = exp2(2);`
  },
  extractBits: {
    keyword: `extractBits`,
    src: `_ = extractBits(1, 2, 3);`
  },
  f32: {
    keyword: `f32`,
    src: `_ = f32(1i);`
  },
  faceForward: {
    keyword: `faceForward`,
    src: `_ = faceForward(vec2(1, 2), vec2(3, 4), vec2(5, 6));`
  },
  firstLeadingBit: {
    keyword: `firstLeadingBit`,
    src: `_ = firstLeadingBit(1);`
  },
  firstTrailingBit: {
    keyword: `firstTrailingBit`,
    src: `_ = firstTrailingBit(1);`
  },
  floor: {
    keyword: `floor`,
    src: `_ = floor(1.2);`
  },
  fma: {
    keyword: `fma`,
    src: `_ = fma(1, 2, 3);`
  },
  fract: {
    keyword: `fract`,
    src: `_ = fract(1);`
  },
  frexp: {
    keyword: `frexp`,
    src: `_ = frexp(1);`
  },
  fwidth: {
    keyword: `fwidth`,
    src: `_ = fwidth(2);`
  },
  fwidthCoarse: {
    keyword: `fwidthCoarse`,
    src: `_ = fwidthCoarse(2);`
  },
  fwidthFine: {
    keyword: `fwidthFine`,
    src: `_ = fwidthFine(2);`
  },
  i32: {
    keyword: `i32`,
    src: `_ = i32(2u);`
  },
  insertBits: {
    keyword: `insertBits`,
    src: `_ = insertBits(1, 2, 3, 4);`
  },
  inverseSqrt: {
    keyword: `inverseSqrt`,
    src: `_ = inverseSqrt(1);`
  },
  ldexp: {
    keyword: `ldexp`,
    src: `_ = ldexp(1, 2);`
  },
  length: {
    keyword: `length`,
    src: `_ = length(1);`
  },
  log: {
    keyword: `log`,
    src: `_ = log(2);`
  },
  log2: {
    keyword: `log2`,
    src: `_ = log2(2);`
  },
  mat2x2_templated: {
    keyword: `mat2x2`,
    src: `_ = mat2x2<f32>(1, 2, 3, 4);`
  },
  mat2x2: {
    keyword: `mat2x2`,
    src: `_ = mat2x2(1, 2, 3, 4);`
  },
  mat2x3_templated: {
    keyword: `mat2x3`,
    src: `_ = mat2x3<f32>(1, 2, 3, 4, 5, 6);`
  },
  mat2x3: {
    keyword: `mat2x3`,
    src: `_ = mat2x3(1, 2, 3, 4, 5, 6);`
  },
  mat2x4_templated: {
    keyword: `mat2x4`,
    src: `_ = mat2x4<f32>(1, 2, 3, 4, 5, 6, 7, 8);`
  },
  mat2x4: {
    keyword: `mat2x4`,
    src: `_ = mat2x4(1, 2, 3, 4, 5, 6, 7, 8);`
  },
  mat3x2_templated: {
    keyword: `mat3x2`,
    src: `_ = mat3x2<f32>(1, 2, 3, 4, 5, 6);`
  },
  mat3x2: {
    keyword: `mat3x2`,
    src: `_ = mat3x2(1, 2, 3, 4, 5, 6);`
  },
  mat3x3_templated: {
    keyword: `mat3x3`,
    src: `_ = mat3x3<f32>(1, 2, 3, 4, 5, 6, 7, 8, 9);`
  },
  mat3x3: {
    keyword: `mat3x3`,
    src: `_ = mat3x3(1, 2, 3, 4, 5, 6, 7, 8, 9);`
  },
  mat3x4_templated: {
    keyword: `mat3x4`,
    src: `_ = mat3x4<f32>(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);`
  },
  mat3x4: {
    keyword: `mat3x4`,
    src: `_ = mat3x4(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);`
  },
  mat4x2_templated: {
    keyword: `mat4x2`,
    src: `_ = mat4x2<f32>(1, 2, 3, 4, 5, 6, 7, 8);`
  },
  mat4x2: {
    keyword: `mat4x2`,
    src: `_ = mat4x2(1, 2, 3, 4, 5, 6, 7, 8);`
  },
  mat4x3_templated: {
    keyword: `mat4x3`,
    src: `_ = mat4x3<f32>(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);`
  },
  mat4x3: {
    keyword: `mat4x3`,
    src: `_ = mat4x3(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);`
  },
  mat4x4_templated: {
    keyword: `mat4x4`,
    src: `_ = mat4x4<f32>(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);`
  },
  mat4x4: {
    keyword: `mat4x4`,
    src: `_ = mat4x4(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);`
  },
  max: {
    keyword: `max`,
    src: `_ = max(1, 2);`
  },
  min: {
    keyword: `min`,
    src: `_ = min(1, 2);`
  },
  mix: {
    keyword: `mix`,
    src: `_ = mix(1, 2, 3);`
  },
  modf: {
    keyword: `modf`,
    src: `_ = modf(1.2);`
  },
  normalize: {
    keyword: `normalize`,
    src: `_ = normalize(vec2(1, 2));`
  },
  pack2x16snorm: {
    keyword: `pack2x16snorm`,
    src: `_ = pack2x16snorm(vec2(1, 2));`
  },
  pack2x16unorm: {
    keyword: `pack2x16unorm`,
    src: `_ = pack2x16unorm(vec2(1, 2));`
  },
  pack2x16float: {
    keyword: `pack2x16float`,
    src: `_ = pack2x16float(vec2(1, 2));`
  },
  pack4x8snorm: {
    keyword: `pack4x8snorm`,
    src: `_ = pack4x8snorm(vec4(1, 2, 3, 4));`
  },
  pack4x8unorm: {
    keyword: `pack4x8unorm`,
    src: `_ = pack4x8unorm(vec4(1, 2, 3, 4));`
  },
  pack4xI8: {
    keyword: `pack4xI8`,
    src: `_ = pack4xI8(vec4(1, 2, 3, 4));`
  },
  pack4xU8: {
    keyword: `pack4xU8`,
    src: `_ = pack4xU8(vec4(1, 2, 3, 4));`
  },
  pack4xI8Clamp: {
    keyword: `pack4xI8Clamp`,
    src: `_ = pack4xI8Clamp(vec4(1, 2, 3, 4));`
  },
  pack4xU8Clamp: {
    keyword: `pack4xU8Clamp`,
    src: `_ = pack4xU8Clamp(vec4(1, 2, 3, 4));`
  },
  pow: {
    keyword: `pow`,
    src: `_ = pow(1, 2);`
  },
  quantizeToF16: {
    keyword: `quantizeToF16`,
    src: `_ = quantizeToF16(1.2);`
  },
  radians: {
    keyword: `radians`,
    src: `_ = radians(1.2);`
  },
  reflect: {
    keyword: `reflect`,
    src: `_ = reflect(vec2(1, 2), vec2(3, 4));`
  },
  refract: {
    keyword: `refract`,
    src: `_ = refract(vec2(1, 1), vec2(2, 2), 3);`
  },
  reverseBits: {
    keyword: `reverseBits`,
    src: `_ = reverseBits(1);`
  },
  round: {
    keyword: `round`,
    src: `_ = round(1.2);`
  },
  saturate: {
    keyword: `saturate`,
    src: `_ = saturate(1);`
  },
  select: {
    keyword: `select`,
    src: `_ = select(1, 2, false);`
  },
  sign: {
    keyword: `sign`,
    src: `_ = sign(1);`
  },
  sin: {
    keyword: `sin`,
    src: `_ = sin(2);`
  },
  sinh: {
    keyword: `sinh`,
    src: `_ = sinh(3);`
  },
  smoothstep: {
    keyword: `smoothstep`,
    src: `_ = smoothstep(1, 2, 3);`
  },
  sqrt: {
    keyword: `sqrt`,
    src: `_ = sqrt(24);`
  },
  step: {
    keyword: `step`,
    src: `_ = step(4, 5);`
  },
  tan: {
    keyword: `tan`,
    src: `_ = tan(2);`
  },
  tanh: {
    keyword: `tanh`,
    src: `_ = tanh(2);`
  },
  transpose: {
    keyword: `transpose`,
    src: `_ = transpose(mat2x2(1, 2, 3, 4));`
  },
  trunc: {
    keyword: `trunc`,
    src: `_ = trunc(2);`
  },
  u32: {
    keyword: `u32`,
    src: `_ = u32(1i);`
  },
  unpack2x16snorm: {
    keyword: `unpack2x16snorm`,
    src: `_ = unpack2x16snorm(2);`
  },
  unpack2x16unorm: {
    keyword: `unpack2x16unorm`,
    src: `_ = unpack2x16unorm(2);`
  },
  unpack2x16float: {
    keyword: `unpack2x16float`,
    src: `_ = unpack2x16float(2);`
  },
  unpack4x8snorm: {
    keyword: `unpack4x8snorm`,
    src: `_ = unpack4x8snorm(4);`
  },
  unpack4x8unorm: {
    keyword: `unpack4x8unorm`,
    src: `_ = unpack4x8unorm(4);`
  },
  unpack4xI8: {
    keyword: `unpack4xI8`,
    src: `_ = unpack4xI8(4);`
  },
  unpack4xU8: {
    keyword: `unpack4xU8`,
    src: `_ = unpack4xU8(4);`
  },
  vec2_templated: {
    keyword: `vec2`,
    src: `_ = vec2<f32>(1, 2);`
  },
  vec2: {
    keyword: `vec2`,
    src: `_ = vec2(1, 2);`
  },
  vec3_templated: {
    keyword: `vec3`,
    src: `_ = vec3<f32>(1, 2, 3);`
  },
  vec3: {
    keyword: `vec3`,
    src: `_ = vec3(1, 2, 3);`
  },
  vec4_templated: {
    keyword: `vec4`,
    src: `_ = vec4<f32>(1, 2, 3, 4);`
  },
  vec4: {
    keyword: `vec4`,
    src: `_ = vec4(1, 2, 3, 4);`
  }
};

g.test('shadow_hides_builtin').
desc(`Test that shadows hide builtins.`).
params((u) =>
u.
combine('inject', ['none', 'function', 'sibling', 'module']).
beginSubcases().
combine('builtin', keysOf(kTests))
).
fn((t) => {
  const data = kTests[t.params.builtin];
  const local = `let ${data.keyword} = 4;`;

  const module_shadow = t.params.inject === 'module' ? `var<private> ${data.keyword} : i32;` : ``;
  const sibling_func = t.params.inject === 'sibling' ? local : ``;
  const func = t.params.inject === 'function' ? local : ``;

  const code = `
struct Data {
  rt_arr: array<i32>,
}
@group(0) @binding(0) var<storage> placeholder: Data;

${module_shadow}

fn sibling() {
  ${sibling_func}
}

@fragment
fn main() -> @location(0) vec4f {
  ${func}
  ${data.src}
  return vec4f(1);
}
    `;

  const pass = t.params.inject === 'none' || t.params.inject === 'sibling';
  t.expectCompileResult(pass, code);
});

const kFloat16Tests = {
  f16: {
    keyword: `f16`,
    src: `_ = f16(2);`
  }
};

g.test('shadow_hides_builtin_f16').
desc(`Test that shadows hide builtins when shader-f16 is enabled.`).
params((u) =>
u.
combine('inject', ['none', 'function', 'sibling', 'module']).
beginSubcases().
combine('builtin', keysOf(kFloat16Tests))
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn((t) => {
  const data = kFloat16Tests[t.params.builtin];
  const local = `let ${data.keyword} = 4;`;

  const module_shadow = t.params.inject === 'module' ? `var<private> ${data.keyword} : f16;` : ``;
  const sibling_func = t.params.inject === 'sibling' ? local : ``;
  const func = t.params.inject === 'function' ? local : ``;

  const code = `
enable f16;

${module_shadow}

fn sibling() {
  ${sibling_func}
}

@vertex
fn vtx() -> @builtin(position) vec4f {
  ${func}
  ${data.src}
  return vec4f(1);
}
    `;
  const pass = t.params.inject === 'none' || t.params.inject === 'sibling';
  t.expectCompileResult(pass, code);
});

const kTextureTypeTests = {
  texture_1d: {
    keyword: `texture_1d`,
    src: `var t: texture_1d<f32>;`
  },
  texture_2d: {
    keyword: `texture_2d`,
    src: `var t: texture_2d<f32>;`
  },
  texture_2d_array: {
    keyword: `texture_2d_array`,
    src: `var t: texture_2d_array<f32>;`
  },
  texture_3d: {
    keyword: `texture_3d`,
    src: `var t: texture_3d<f32>;`
  },
  texture_cube: {
    keyword: `texture_cube`,
    src: `var t: texture_cube<f32>;`
  },
  texture_cube_array: {
    keyword: `texture_cube_array`,
    src: `var t: texture_cube_array<f32>;`
  },
  texture_multisampled_2d: {
    keyword: `texture_multisampled_2d`,
    src: `var t: texture_multisampled_2d<f32>;`
  },
  texture_depth_multisampled_2d: {
    keyword: `texture_depth_multisampled_2d`,
    src: `var t: texture_depth_multisampled_2d;`
  },
  texture_external: {
    keyword: `texture_external`,
    src: `var t: texture_external;`
  },
  texture_storage_1d: {
    keyword: `texture_storage_1d`,
    src: `var t: texture_storage_1d<rgba8unorm, read_write>;`
  },
  texture_storage_2d: {
    keyword: `texture_storage_2d`,
    src: `var t: texture_storage_2d<rgba8unorm, read_write>;`
  },
  texture_storage_2d_array: {
    keyword: `texture_storage_2d_array`,
    src: `var t: texture_storage_2d_array<rgba8unorm, read_write>;`
  },
  texture_storage_3d: {
    keyword: `texture_storage_3d`,
    src: `var t: texture_storage_3d<rgba8unorm, read_write>;`
  },
  texture_depth_2d: {
    keyword: `texture_depth_2d`,
    src: `var t: texture_depth_2d;`
  },
  texture_depth_2d_array: {
    keyword: `texture_depth_2d_array`,
    src: `var t: texture_depth_2d_array;`
  },
  texture_depth_cube: {
    keyword: `texture_depth_cube`,
    src: `var t: texture_depth_cube;`
  },
  texture_depth_cube_array: {
    keyword: `texture_depth_cube_array`,
    src: `var t: texture_depth_cube_array;`
  },
  sampler: {
    keyword: `sampler`,
    src: `var s: sampler;`
  },
  sampler_comparison: {
    keyword: `sampler_comparison`,
    src: `var s: sampler_comparison;`
  }
};

g.test('shadow_hides_builtin_handle_type').
desc(`Test that shadows hide builtins when handle address space types are used.`).
params((u) =>
u.
combine('inject', ['none', 'function', 'module']).
beginSubcases().
combine('builtin', keysOf(kTextureTypeTests))
).
fn((t) => {
  const data = kTextureTypeTests[t.params.builtin];
  const local = `let ${data.keyword} = 4;`;

  const module_shadow = t.params.inject === 'module' ? `var<private> ${data.keyword} : f32;` : ``;
  const func = t.params.inject === 'function' ? local : ``;

  const code = `
${module_shadow}
@group(0) @binding(0) ${data.src}

fn func() {
  ${func}
}
    `;
  const pass = t.params.inject === 'none' || t.params.inject === 'function';
  t.expectCompileResult(pass, code);
});

const kTextureTests = {
  textureDimensions: {
    keyword: `textureDimensions`,
    src: `_ = textureDimensions(t_2d);`
  },
  textureGather: {
    keyword: `textureGather`,
    src: `_ = textureGather(1, t_2d, s, vec2(1, 2));`
  },
  textureGatherCompare: {
    keyword: `textureGatherCompare`,
    src: `_ = textureGatherCompare(t_2d_depth, sc, vec2(1, 2), 3);`
  },
  textureLoad: {
    keyword: `textureLoad`,
    src: `_ = textureLoad(t_2d, vec2(1, 2), 1);`
  },
  textureNumLayers: {
    keyword: `textureNumLayers`,
    src: `_ = textureNumLayers(t_2d_array);`
  },
  textureNumLevels: {
    keyword: `textureNumLevels`,
    src: `_ = textureNumLevels(t_2d);`
  },
  textureNumSamples: {
    keyword: `textureNumSamples`,
    src: `_ = textureNumSamples(t_2d_ms);`
  },
  textureSample: {
    keyword: `textureSample`,
    src: `_ = textureSample(t_2d, s, vec2(1, 2));`
  },
  textureSampleBias: {
    keyword: `textureSampleBias`,
    src: `_ = textureSampleBias(t_2d, s, vec2(1, 2), 2);`
  },
  textureSampleCompare: {
    keyword: `textureSampleCompare`,
    src: `_ = textureSampleCompare(t_2d_depth, sc, vec2(1, 2), 2);`
  },
  textureSampleCompareLevel: {
    keyword: `textureSampleCompareLevel`,
    src: `_ = textureSampleCompareLevel(t_2d_depth, sc, vec2(1, 2), 3, vec2(1, 2));`
  },
  textureSampleGrad: {
    keyword: `textureSampleGrad`,
    src: `_ = textureSampleGrad(t_2d, s, vec2(1, 2), vec2(1, 2), vec2(1, 2));`
  },
  textureSampleLevel: {
    keyword: `textureSampleLevel`,
    src: `_ = textureSampleLevel(t_2d, s, vec2(1, 2), 3);`
  },
  textureSampleBaseClampToEdge: {
    keyword: `textureSampleBaseClampToEdge`,
    src: `_ = textureSampleBaseClampToEdge(t_2d, s, vec2(1, 2));`
  }
};

g.test('shadow_hides_builtin_texture').
desc(`Test that shadows hide texture builtins.`).
params((u) =>
u.
combine('inject', ['none', 'function', 'sibling', 'module']).
beginSubcases().
combine('builtin', keysOf(kTextureTests))
).
fn((t) => {
  const data = kTextureTests[t.params.builtin];
  const local = `let ${data.keyword} = 4;`;

  const module_shadow = t.params.inject === 'module' ? `var<private> ${data.keyword} : i32;` : ``;
  const sibling_func = t.params.inject === 'sibling' ? local : ``;
  const func = t.params.inject === 'function' ? local : ``;

  const code = `
@group(0) @binding(0) var t_2d: texture_2d<f32>;
@group(0) @binding(1) var t_2d_depth: texture_depth_2d;
@group(0) @binding(2) var t_2d_array: texture_2d_array<f32>;
@group(0) @binding(3) var t_2d_ms: texture_multisampled_2d<f32>;

@group(1) @binding(0) var s: sampler;
@group(1) @binding(1) var sc: sampler_comparison;

${module_shadow}

fn sibling() {
  ${sibling_func}
}

@fragment
fn main() -> @location(0) vec4f {
  ${func}
  ${data.src}
  return vec4f(1);
}
    `;

  const pass = t.params.inject === 'none' || t.params.inject === 'sibling';
  t.expectCompileResult(pass, code);
});

g.test('shadow_hides_builtin_atomic_type').
desc(`Test that shadows hide builtins when atomic types are used.`).
params((u) => u.combine('inject', ['none', 'function', 'module']).beginSubcases()).
fn((t) => {
  const local = `let atomic = 4;`;
  const module_shadow = t.params.inject === 'module' ? `var<private> atomic: i32;` : ``;
  const func = t.params.inject === 'function' ? local : ``;

  const code = `
${module_shadow}

var<workgroup> val: atomic<i32>;

fn func() {
  ${func}
}
    `;
  const pass = t.params.inject === 'none' || t.params.inject === 'function';
  t.expectCompileResult(pass, code);
});

const kAtomicTests = {
  atomicLoad: {
    keyword: `atomicLoad`,
    src: `_ = atomicLoad(&a);`
  },
  atomicStore: {
    keyword: `atomicStore`,
    src: `atomicStore(&a, 1);`
  },
  atomicAdd: {
    keyword: `atomicAdd`,
    src: `_ = atomicAdd(&a, 1);`
  },
  atomicSub: {
    keyword: `atomicSub`,
    src: `_ = atomicSub(&a, 1);`
  },
  atomicMax: {
    keyword: `atomicMax`,
    src: `_ = atomicMax(&a, 1);`
  },
  atomicMin: {
    keyword: `atomicMin`,
    src: `_ = atomicMin(&a, 1);`
  },
  atomicAnd: {
    keyword: `atomicAnd`,
    src: `_ = atomicAnd(&a, 1);`
  },
  atomicOr: {
    keyword: `atomicOr`,
    src: `_ = atomicOr(&a, 1);`
  },
  atomicXor: {
    keyword: `atomicXor`,
    src: `_ = atomicXor(&a, 1);`
  }
};

g.test('shadow_hides_builtin_atomic').
desc(`Test that shadows hide builtin atomic methods.`).
params((u) =>
u.
combine('inject', ['none', 'function', 'sibling', 'module']).
beginSubcases().
combine('builtin', keysOf(kAtomicTests))
).
fn((t) => {
  const data = kAtomicTests[t.params.builtin];
  const local = `let ${data.keyword} = 4;`;

  const module_shadow = t.params.inject === 'module' ? `var<private> ${data.keyword} : i32;` : ``;
  const sibling_func = t.params.inject === 'sibling' ? local : ``;
  const func = t.params.inject === 'function' ? local : ``;

  const code = `
var<workgroup> a: atomic<i32>;

${module_shadow}

fn sibling() {
  ${sibling_func}
}

@compute @workgroup_size(1)
fn main() {
  ${func}
  ${data.src}
}
    `;

  const pass = t.params.inject === 'none' || t.params.inject === 'sibling';
  t.expectCompileResult(pass, code);
});

const kBarrierTests = {
  storageBarrier: {
    keyword: `storageBarrier`,
    src: `storageBarrier();`
  },
  textureBarrier: {
    keyword: `textureBarrier`,
    src: `textureBarrier();`
  },
  workgroupBarrier: {
    keyword: `workgroupBarrier`,
    src: `workgroupBarrier();`
  },
  workgroupUniformLoad: {
    keyword: `workgroupUniformLoad`,
    src: `_ = workgroupUniformLoad(&u);`
  }
};

g.test('shadow_hides_builtin_barriers').
desc(`Test that shadows hide builtin barrier methods.`).
params((u) =>
u.
combine('inject', ['none', 'function', 'sibling', 'module']).
beginSubcases().
combine('builtin', keysOf(kBarrierTests))
).
fn((t) => {
  const data = kBarrierTests[t.params.builtin];
  const local = `let ${data.keyword} = 4;`;

  const module_shadow = t.params.inject === 'module' ? `var<private> ${data.keyword} : i32;` : ``;
  const sibling_func = t.params.inject === 'sibling' ? local : ``;
  const func = t.params.inject === 'function' ? local : ``;

  const code = `
var<workgroup> u: u32;

${module_shadow}

fn sibling() {
  ${sibling_func}
}

@compute @workgroup_size(1)
fn main() {
  ${func}
  ${data.src}
}
    `;

  const pass = t.params.inject === 'none' || t.params.inject === 'sibling';
  t.expectCompileResult(pass, code);
});

const kAccessModeTests = {
  read: {
    keyword: `read`,
    src: `var<storage, read> a: i32;`
  },
  read_write: {
    keyword: `read_write`,
    src: `var<storage, read_write> a: i32;`
  },
  write: {
    keyword: `write`,
    src: `var t: texture_storage_1d<rgba8unorm, write>;`
  }
};

g.test('shadow_hides_access_mode').
desc(`Test that shadows hide access modes.`).
params((u) =>
u.
combine('inject', ['none', 'function', 'module']).
beginSubcases().
combine('builtin', keysOf(kAccessModeTests))
).
fn((t) => {
  const data = kAccessModeTests[t.params.builtin];
  const local = `let ${data.keyword} = 4;`;

  const module_shadow = t.params.inject === 'module' ? `var<private> ${data.keyword} : i32;` : ``;
  const func = t.params.inject === 'function' ? local : ``;

  const code = `
${module_shadow}

@group(0) @binding(0) ${data.src}

@compute @workgroup_size(1)
fn main() {
  ${func}
}
    `;

  const pass = t.params.inject === 'none' || t.params.inject === 'function';
  t.expectCompileResult(pass, code);
});