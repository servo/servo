/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for @must_use`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kMustUseDeclarations = {
  var: {
    code: `@must_use @group(0) @binding(0)
    var<storage> x : array<u32>;`,
    valid: false,
  },
  function_no_return: {
    code: `@must_use fn foo() { }`,
    valid: false,
  },
  function_scalar_return: {
    code: `@must_use fn foo() -> u32 { return 0; }`,
    valid: true,
  },
  function_struct_return: {
    code: `struct S { x : u32 }
    @must_use fn foo() -> S { return S(); }`,
    valid: true,
  },
  function_var: {
    code: `fn foo() { @must_use var x = 0; }`,
    valid: false,
  },
  function_call: {
    code: `fn bar() -> u32 { return 0; }
    fn foo() { @must_use bar(); }`,
    valid: false,
  },
  function_parameter: {
    code: `fn foo(@must_use param : u32) -> u32 { return param; }`,
    valid: false,
  },
  empty_parameter: {
    code: `@must_use() fn foo() -> u32 { return 0; }`,
    valid: false,
  },
  parameter: {
    code: `@must_use(0) fn foo() -> u32 { return 0; }`,
    valid: false,
  },
};

g.test('declaration')
  .desc(`Validate attribute can only be applied to a function declaration with a return type`)
  .params(u => u.combine('test', keysOf(kMustUseDeclarations)))
  .fn(t => {
    const test = kMustUseDeclarations[t.params.test];
    t.expectCompileResult(test.valid, test.code);
  });

const kMustUseCalls = {
  phony: `_ = bar();`,
  let: `let tmp = bar();`,
  var: `var tmp = bar();`,
  condition: `if bar() == 0 { }`,
  param: `baz(bar());`,
  statement: `bar();`,
};

g.test('call')
  .desc(`Validate that a call to must_use function cannot be the whole function call statement`)
  .params(u => u.combine('use', ['@must_use', '']).combine('call', keysOf(kMustUseCalls)))
  .fn(t => {
    const test = kMustUseCalls[t.params.call];
    const code = `
    fn baz(param : u32) { }
    ${t.params.use} fn bar() -> u32 { return 0; }
    fn foo() {
      ${test}
    }`;
    const res = t.params.call !== 'statement' || t.params.use === '';
    t.expectCompileResult(res, code);
  });

const kMustUseBuiltinCalls = {
  // Type constructors
  u32: `u32()`,
  i32: `i32(0)`,
  struct: `S()`,
  // Reinterpretation
  bitcast: `bitcast<f32>(8u)`,
  // Logical
  all: `all(vec2<bool>(true))`,
  any: `any(vec2<bool>(true))`,
  select: `select(0i, 1i, true)`,
  // Array
  arrayLength: `arrayLength(&storage_var)`,
  // Numeric
  abs: `abs(0.5)`,
  acos: `acos(0.5)`,
  acosh: `acosh(1.0)`,
  asin: `asin(0.5)`,
  asinh: `asinh(0.5)`,
  atan: `atan(0.5)`,
  atanh: `atanh(0.5)`,
  atan2: `atan2(0.5, 0.5)`,
  ceil: `ceil(0.5)`,
  clamp: `clamp(0.5, 0.1, 1.0)`,
  cos: `cos(0.5)`,
  cosh: `cosh(0.5)`,
  countLeadingZeros: `countLeadingZeros(0)`,
  countOneBits: `countOneBits(0)`,
  countTrailingZeros: `countTrailingZeros(0)`,
  cross: `cross(vec3f(), vec3f())`,
  degrees: `degrees(0.5)`,
  determinant: `determinant(mat2x2f())`,
  distance: `distance(0.5, 0.5)`,
  dot: `dot(vec2f(0.5, 0.5), vec2f(0.5, 0.5))`,
  exp: `exp(0.5)`,
  exp2: `exp2(0.5)`,
  extractBits: `extractBits(0, 0, 1)`,
  faceForward: `faceForward(vec2f(), vec2f(), vec2f())`,
  firstLeadingBit: `firstLeadingBit(0)`,
  firstTrailingBit: `firstTrailingBit(0)`,
  floor: `floor(0.5)`,
  fma: `fma(0.5, 0.5, 0.5)`,
  fract: `fract(0.5)`,
  frexp: `frexp(0.5)`,
  insertBits: `insertBits(0, 0, 0, 1)`,
  inverseSqrt: `inverseSqrt(0.5)`,
  ldexp: `ldexp(0.5, 1)`,
  length: `length(0.5)`,
  log: `log(0.5)`,
  log2: `log2(0.5)`,
  max: `max(0, 0)`,
  min: `min(0, 0)`,
  mix: `mix(0.5, 0.5, 0.5)`,
  modf: `modf(0.5)`,
  normalize: `normalize(vec2f(0.5, 0.5))`,
  pow: `pow(0.5, 0.5)`,
  quantizeToF16: `quantizeToF16(0.5)`,
  radians: `radians(0.5)`,
  reflect: `reflect(vec2f(0.5, 0.5), vec2f(0.5, 0.5))`,
  refract: `refract(vec2f(0.5, 0.5), vec2f(0.5, 0.5), 0.5)`,
  reverseBits: `reverseBits(0)`,
  round: `round(0.5)`,
  saturate: `saturate(0.5)`,
  sign: `sign(0.5)`,
  sin: `sin(0.5)`,
  sinh: `sinh(0.5)`,
  smoothstep: `smoothstep(0.1, 1.0, 0.5)`,
  sqrt: `sqrt(0.5)`,
  step: `step(0.1, 0.5)`,
  tan: `tan(0.5)`,
  tanh: `tanh(0.5)`,
  transpose: `transpose(mat2x2f())`,
  trunc: `trunc(0.5)`,
  // Derivative
  dpdx: `dpdx(0.5)`,
  dpdxCoarse: `dpdxCoarse(0.5)`,
  dpdxFine: `dpdxFine(0.5)`,
  dpdy: `dpdy(0.5)`,
  dpdyCoarse: `dpdyCoarse(0.5)`,
  dpdyFine: `dpdyFine(0.5)`,
  fwidth: `fwidth(0.5)`,
  fwidthCoarse: `fwidthCoarse(0.5)`,
  fwidthFine: `fwidthFine(0.5)`,
  // Texture
  textureDimensions: `textureDimensions(tex_2d)`,
  textureGather: `textureGather(0, tex_2d, s, vec2f(0,0))`,
  textureGatherCompare: `textureGatherCompare(tex_depth_2d, s_comp, vec2f(0,0), 0)`,
  textureLoad: `textureLoad(tex_2d, vec2i(0,0), 0)`,
  textureNumLayers: `textureNumLayers(tex_array_2d)`,
  textureNumLevels: `textureNumLevels(tex_2d)`,
  textureNumSamples: `textureNumSamples(tex_multi_2d)`,
  textureSample: `textureSample(tex_2d, s, vec2f(0,0))`,
  textureSampleBias: `textureSampleBias(tex_2d, s, vec2f(0,0), 0)`,
  textureSampleCompare: `textureSampleCompare(tex_depth_2d, s_comp, vec2f(0,0), 0)`,
  textureSampleCompareLevel: `textureSampleCompareLevel(tex_depth_2d, s_comp, vec2f(0,0), 0)`,
  textureSampleGrad: `textureSampleGrad(tex_2d, s, vec2f(0,0), vec2f(0,0), vec2f(0,0))`,
  textureSampleLevel: `textureSampleLevel(tex_2d, s, vec2f(0,0), 0)`,
  textureSampleBaseClampToEdge: `textureSampleBaseClampToEdge(tex_2d, s, vec2f(0,0))`,
  // Data Packing
  pack4x8snorm: `pack4x8snorm(vec4f())`,
  pack4x8unorm: `pack4x8unorm(vec4f())`,
  pack2x16snorm: `pack2x16snorm(vec2f())`,
  pack2x16unorm: `pack2x16unorm(vec2f())`,
  pack2x16float: `pack2x16float(vec2f())`,
  // Data Unpacking
  unpack4x8snorm: `unpack4x8snorm(0)`,
  unpack4x8unorm: `unpack4x8unorm(0)`,
  unpack2x16snorm: `unpack2x16snorm(0)`,
  unpack2x16unorm: `unpack2x16unorm(0)`,
  unpack2x16float: `unpack2x16float(0)`,
  // Synchronization
  workgroupUniformLoad: `workgroupUniformLoad(&wg_var)`,
};

g.test('builtin_must_use')
  .desc(`Validate must_use built-in functions`)
  .params(u => u.combine('call', keysOf(kMustUseBuiltinCalls)).combine('use', [true, false]))
  .fn(t => {
    let call = kMustUseBuiltinCalls[t.params.call];
    if (t.params.use) {
      call = `_ = ${call}`;
    }
    const code = `
struct S {
  x : u32
}

@group(0) @binding(0)
var<storage> storage_var : array<u32>;
@group(0) @binding(1)
var tex_2d : texture_2d<f32>;
@group(0) @binding(2)
var s : sampler;
@group(0) @binding(3)
var tex_depth_2d : texture_depth_2d;
@group(0) @binding(4)
var s_comp : sampler_comparison;
@group(0) @binding(5)
var tex_storage_2d : texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(6)
var tex_multi_2d : texture_multisampled_2d<f32>;
@group(0) @binding(7)
var tex_array_2d : texture_2d_array<f32>;

var<workgroup> wg_var : u32;

fn foo() {
  ${call};
}`;

    t.expectCompileResult(t.params.use, code);
  });

const kNoMustUseBuiltinCalls = {
  atomicLoad: `atomicLoad(&a)`,
  atomicAdd: `atomicAdd(&a, 0)`,
  atomicSub: `atomicSub(&a, 0)`,
  atomicMax: `atomicMax(&a, 0)`,
  atomicMin: `atomicMin(&a, 0)`,
  atomicAnd: `atomicAnd(&a, 0)`,
  atomicOr: `atomicOr(&a, 0)`,
  atomicXor: `atomicXor(&a, 0)`,
  atomicExchange: `atomicExchange(&a, 0)`,
  atomicCompareExchangeWeak: `atomicCompareExchangeWeak(&a, 0, 0)`,
};

g.test('builtin_no_must_use')
  .desc(`Validate built-in functions without must_use`)
  .params(u => u.combine('call', keysOf(kNoMustUseBuiltinCalls)).combine('use', [true, false]))
  .fn(t => {
    let call = kNoMustUseBuiltinCalls[t.params.call];
    if (t.params.use) {
      call = `_ = ${call}`;
    }
    const code = `
var<workgroup> a : atomic<u32>;

fn foo() {
  ${call};
}`;

    t.expectCompileResult(true, code);
  });
