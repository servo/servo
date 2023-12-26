/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for vector accesses
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kCases = {
  // indexing with literal
  literal_0: { wgsl: 'let r : T = v[0];', ok: true },
  literal_1: { wgsl: 'let r : T = v[1];', ok: true },
  literal_2: { wgsl: 'let r : T = v[2];', ok: (width) => width > 2 },
  literal_3: { wgsl: 'let r : T = v[3];', ok: (width) => width > 3 },
  literal_0i: { wgsl: 'let r : T = v[0i];', ok: true },
  literal_1i: { wgsl: 'let r : T = v[1i];', ok: true },
  literal_2i: { wgsl: 'let r : T = v[2i];', ok: (width) => width > 2 },
  literal_3i: { wgsl: 'let r : T = v[3i];', ok: (width) => width > 3 },
  literal_0u: { wgsl: 'let r : T = v[0u];', ok: true },
  literal_1u: { wgsl: 'let r : T = v[1u];', ok: true },
  literal_2u: { wgsl: 'let r : T = v[2u];', ok: (width) => width > 2 },
  literal_3u: { wgsl: 'let r : T = v[3u];', ok: (width) => width > 3 },

  // indexing with 'const' variable
  const_0: { wgsl: 'const i = 0; let r : T = v[i];', ok: true },
  const_1: { wgsl: 'const i = 1; let r : T = v[i];', ok: true },
  const_2: { wgsl: 'const i = 2; let r : T = v[i];', ok: (width) => width > 2 },
  const_3: { wgsl: 'const i = 3; let r : T = v[i];', ok: (width) => width > 3 },
  const_0i: { wgsl: 'const i = 0i; let r : T = v[i];', ok: true },
  const_1i: { wgsl: 'const i = 1i; let r : T = v[i];', ok: true },
  const_2i: { wgsl: 'const i = 2i; let r : T = v[i];', ok: (width) => width > 2 },
  const_3i: { wgsl: 'const i = 3i; let r : T = v[i];', ok: (width) => width > 3 },
  const_0u: { wgsl: 'const i = 0u; let r : T = v[i];', ok: true },
  const_1u: { wgsl: 'const i = 1u; let r : T = v[i];', ok: true },
  const_2u: { wgsl: 'const i = 2u; let r : T = v[i];', ok: (width) => width > 2 },
  const_3u: { wgsl: 'const i = 3u; let r : T = v[i];', ok: (width) => width > 3 },

  // indexing with 'let' variable
  let_0: { wgsl: 'let i = 0; let r : T = v[i];', ok: true },
  let_1: { wgsl: 'let i = 1; let r : T = v[i];', ok: true },
  let_2: { wgsl: 'let i = 2; let r : T = v[i];', ok: true },
  let_3: { wgsl: 'let i = 3; let r : T = v[i];', ok: true },
  let_0i: { wgsl: 'let i = 0i; let r : T = v[i];', ok: true },
  let_1i: { wgsl: 'let i = 1i; let r : T = v[i];', ok: true },
  let_2i: { wgsl: 'let i = 2i; let r : T = v[i];', ok: true },
  let_3i: { wgsl: 'let i = 3i; let r : T = v[i];', ok: true },
  let_0u: { wgsl: 'let i = 0u; let r : T = v[i];', ok: true },
  let_1u: { wgsl: 'let i = 1u; let r : T = v[i];', ok: true },
  let_2u: { wgsl: 'let i = 2u; let r : T = v[i];', ok: true },
  let_3u: { wgsl: 'let i = 3u; let r : T = v[i];', ok: true },

  // indexing with 'var' variable
  var_0: { wgsl: 'var i = 0; let r : T = v[i];', ok: true },
  var_1: { wgsl: 'var i = 1; let r : T = v[i];', ok: true },
  var_2: { wgsl: 'var i = 2; let r : T = v[i];', ok: true },
  var_3: { wgsl: 'var i = 3; let r : T = v[i];', ok: true },
  var_0i: { wgsl: 'var i = 0i; let r : T = v[i];', ok: true },
  var_1i: { wgsl: 'var i = 1i; let r : T = v[i];', ok: true },
  var_2i: { wgsl: 'var i = 2i; let r : T = v[i];', ok: true },
  var_3i: { wgsl: 'var i = 3i; let r : T = v[i];', ok: true },
  var_0u: { wgsl: 'var i = 0u; let r : T = v[i];', ok: true },
  var_1u: { wgsl: 'var i = 1u; let r : T = v[i];', ok: true },
  var_2u: { wgsl: 'var i = 2u; let r : T = v[i];', ok: true },
  var_3u: { wgsl: 'var i = 3u; let r : T = v[i];', ok: true },

  // indexing with const expression
  const_expr_0: { wgsl: 'let r : T = v[0 / 2];', ok: true },
  const_expr_1: { wgsl: 'let r : T = v[2 / 2];', ok: true },
  const_expr_2: { wgsl: 'let r : T = v[4 / 2];', ok: (width) => width > 2 },
  const_expr_3: { wgsl: 'let r : T = v[6 / 2];', ok: (width) => width > 3 },
  const_expr_2_via_trig: {
    wgsl: 'let r : T = v[i32(tan(1.10714872) + 0.5)];',
    ok: (width) => width > 2
  },
  const_expr_3_via_trig: {
    wgsl: 'let r : T = v[u32(tan(1.24904577) + 0.5)];',
    ok: (width) => width > 3
  },
  const_expr_2_via_vec2: {
    wgsl: 'let r : T = v[vec2(3, 2)[1]];',
    ok: (width) => width > 2
  },
  const_expr_3_via_vec2: {
    wgsl: 'let r : T = v[vec2(3, 2).x];',
    ok: (width) => width > 3
  },
  const_expr_2_via_vec2u: {
    wgsl: 'let r : T = v[vec2u(3, 2)[1]];',
    ok: (width) => width > 2
  },
  const_expr_3_via_vec2i: {
    wgsl: 'let r : T = v[vec2i(3, 2).x];',
    ok: (width) => width > 3
  },
  const_expr_2_via_array: {
    wgsl: 'let r : T = v[array<i32, 2>(3, 2)[1]];',
    ok: (width) => width > 2
  },
  const_expr_3_via_array: {
    wgsl: 'let r : T = v[array<i32, 2>(3, 2)[0]];',
    ok: (width) => width > 3
  },
  const_expr_2_via_struct: {
    wgsl: 'let r : T = v[S(2).i];',
    ok: (width) => width > 2
  },
  const_expr_3_via_struct: {
    wgsl: 'let r : T = v[S(3).i];',
    ok: (width) => width > 3
  },

  // single element convenience name accesses
  x: { wgsl: 'let r : T = v.x;', ok: true },
  y: { wgsl: 'let r : T = v.y;', ok: true },
  z: { wgsl: 'let r : T = v.z;', ok: (width) => width > 2 },
  w: { wgsl: 'let r : T = v.w;', ok: (width) => width > 3 },
  r: { wgsl: 'let r : T = v.r;', ok: true },
  g: { wgsl: 'let r : T = v.g;', ok: true },
  b: { wgsl: 'let r : T = v.b;', ok: (width) => width > 2 },
  a: { wgsl: 'let r : T = v.a;', ok: (width) => width > 3 },

  // swizzles
  xy: { wgsl: 'let r : vec2<T> = v.xy;', ok: true },
  yx: { wgsl: 'let r : vec2<T> = v.yx;', ok: true },
  xyx: { wgsl: 'let r : vec3<T> = v.xyx;', ok: true },
  xyz: { wgsl: 'let r : vec3<T> = v.xyz;', ok: (width) => width > 2 },
  zyx: { wgsl: 'let r : vec3<T> = v.zyx;', ok: (width) => width > 2 },
  xyxy: { wgsl: 'let r : vec4<T> = v.xyxy;', ok: true },
  xyxz: { wgsl: 'let r : vec4<T> = v.xyxz;', ok: (width) => width > 2 },
  xyzw: { wgsl: 'let r : vec4<T> = v.xyzw;', ok: (width) => width > 3 },
  yxwz: { wgsl: 'let r : vec4<T> = v.yxwz;', ok: (width) => width > 3 },
  rg: { wgsl: 'let r : vec2<T> = v.rg;', ok: true },
  gr: { wgsl: 'let r : vec2<T> = v.gr;', ok: true },
  rgg: { wgsl: 'let r : vec3<T> = v.rgg;', ok: true },
  rgb: { wgsl: 'let r : vec3<T> = v.rgb;', ok: (width) => width > 2 },
  grb: { wgsl: 'let r : vec3<T> = v.grb;', ok: (width) => width > 2 },
  rgbr: { wgsl: 'let r : vec4<T> = v.rgbr;', ok: (width) => width > 2 },
  rgba: { wgsl: 'let r : vec4<T> = v.rgba;', ok: (width) => width > 3 },
  gbra: { wgsl: 'let r : vec4<T> = v.gbra;', ok: (width) => width > 3 },

  // swizzle chains
  xy_yx: { wgsl: 'let r : vec2<T> = v.xy.yx;', ok: true },
  xyx_xxy: { wgsl: 'let r : vec3<T> = v.xyx.xxy;', ok: true },
  xyz_zyx: { wgsl: 'let r : vec3<T> = v.xyz.zyx;', ok: (width) => width > 2 },
  xyxy_rrgg: { wgsl: 'let r : vec4<T> = v.xyxy.rrgg;', ok: true },
  rbrg_xyzw: { wgsl: 'let r : vec4<T> = v.rbrg.xyzw;', ok: (width) => width > 2 },
  xyxz_rbg_yx: { wgsl: 'let r : vec2<T> = v.xyxz.rbg.yx;', ok: (width) => width > 2 },
  wxyz_bga_xy: { wgsl: 'let r : vec2<T> = v.wxyz.bga.xy;', ok: (width) => width > 3 },

  // error: invalid convenience letterings
  xq: { wgsl: 'let r : vec2<T> = v.xq;', ok: false },
  py: { wgsl: 'let r : vec2<T> = v.py;', ok: false },

  // error: mixed convenience letterings
  xg: { wgsl: 'let r : vec2<T> = v.xg;', ok: false },
  ryb: { wgsl: 'let r : vec3<T> = v.ryb;', ok: false },
  xgza: { wgsl: 'let r : vec4<T> = v.xgza;', ok: false },

  // error: too many swizzle elements
  xxxxx: { wgsl: 'let r = v.xxxxx;', ok: false },
  rrrrr: { wgsl: 'let r = v.rrrrr;', ok: false },
  yxwxy: { wgsl: 'let r = v.yxwxy;', ok: false },
  rgbar: { wgsl: 'let r = v.rgbar;', ok: false },

  // error: invalid index value
  literal_5: { wgsl: 'let r : T = v[5];', ok: false },
  literal_minus_1: { wgsl: 'let r : T = v[-1];', ok: false },

  // error: invalid index type
  float_idx: { wgsl: 'let r : T = v[1.0];', ok: false },
  bool_idx: { wgsl: 'let r : T = v[true];', ok: false },
  array_idx: { wgsl: 'let r : T = v[array<i32, 2>()];', ok: false }
};

g.test('vector').
desc('Tests validation of vector indexed and swizzles').
params((u) =>
u.
combine('case', keysOf(kCases)) //
.combine('vector_decl', ['const', 'let', 'var', 'param']).
combine('vector_width', [2, 3, 4]).
combine('element_type', ['i32', 'u32', 'f32', 'f16', 'bool'])
).
beforeAllSubcases((t) => {
  if (t.params.element_type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const c = kCases[t.params.case];
  const enables = t.params.element_type === 'f16' ? 'enable f16;' : '';
  const prefix = `${enables}

alias T = ${t.params.element_type};

struct S {
  i : i32,
}

@compute @workgroup_size(1)
`;
  const code =
  t.params.vector_decl === 'param' ?
  `${prefix}
fn main() {
  F(vec${t.params.vector_width}<T>());
}

fn F(v : vec${t.params.vector_width}<T>) {
  ${c.wgsl}
}
` :
  `${prefix}
fn main() {
  ${t.params.vector_decl} v = vec${t.params.vector_width}<T>();
  ${c.wgsl}
}
`;
  const pass = typeof c.ok === 'function' ? c.ok(t.params.vector_width) : c.ok;
  t.expectCompileResult(pass, code);
});