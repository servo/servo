/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for vector accesses
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../common/util/data_tables.js';
import { Type, isConvertible } from '../../../../util/conversion.js';
import { ShaderValidationTest } from '../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kConcreteCases = {
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

const kAbstractCases = {
  // indexing with literal
  literal_0: { wgsl: 'const r = V[0];', result_width: 1, ok: true },
  literal_1: { wgsl: 'const r = V[1];', result_width: 1, ok: true },
  literal_2: { wgsl: 'const r = V[2];', result_width: 1, ok: (width) => width > 2 },
  literal_3: { wgsl: 'const r = V[3];', result_width: 1, ok: (width) => width > 3 },
  literal_0i: { wgsl: 'const r = V[0i];', result_width: 1, ok: true },
  literal_1i: { wgsl: 'const r = V[1i];', result_width: 1, ok: true },
  literal_2i: { wgsl: 'const r = V[2i];', result_width: 1, ok: (width) => width > 2 },
  literal_3i: { wgsl: 'const r = V[3i];', result_width: 1, ok: (width) => width > 3 },
  literal_0u: { wgsl: 'const r = V[0u];', result_width: 1, ok: true },
  literal_1u: { wgsl: 'const r = V[1u];', result_width: 1, ok: true },
  literal_2u: { wgsl: 'const r = V[2u];', result_width: 1, ok: (width) => width > 2 },
  literal_3u: { wgsl: 'const r = V[3u];', result_width: 1, ok: (width) => width > 3 },

  // indexing with 'const' variable
  const_0: { wgsl: 'const i = 0; const r = V[i];', result_width: 1, ok: true },
  const_1: { wgsl: 'const i = 1; const r = V[i];', result_width: 1, ok: true },
  const_2: {
    wgsl: 'const i = 2; const r = V[i];',
    result_width: 1,
    ok: (width) => width > 2
  },
  const_3: {
    wgsl: 'const i = 3; const r = V[i];',
    result_width: 1,
    ok: (width) => width > 3
  },
  const_0i: { wgsl: 'const i = 0i; const r = V[i];', result_width: 1, ok: true },
  const_1i: { wgsl: 'const i = 1i; const r = V[i];', result_width: 1, ok: true },
  const_2i: {
    wgsl: 'const i = 2i; const r = V[i];',
    result_width: 1,
    ok: (width) => width > 2
  },
  const_3i: {
    wgsl: 'const i = 3i; const r = V[i];',
    result_width: 1,
    ok: (width) => width > 3
  },
  const_0u: { wgsl: 'const i = 0u; const r = V[i];', result_width: 1, ok: true },
  const_1u: { wgsl: 'const i = 1u; const r = V[i];', result_width: 1, ok: true },
  const_2u: {
    wgsl: 'const i = 2u; const r = V[i];',
    result_width: 1,
    ok: (width) => width > 2
  },
  const_3u: {
    wgsl: 'const i = 3u; const r = V[i];',
    result_width: 1,
    ok: (width) => width > 3
  },

  // indexing with 'let' variable
  let_0: { wgsl: 'let i = 0; const r = V[i];', ok: false },
  let_1: { wgsl: 'let i = 1; const r = V[i];', ok: false },
  let_2: { wgsl: 'let i = 2; const r = V[i];', ok: false },
  let_3: { wgsl: 'let i = 3; const r = V[i];', ok: false },
  let_0i: { wgsl: 'let i = 0i; const r = V[i];', ok: false },
  let_1i: { wgsl: 'let i = 1i; const r = V[i];', ok: false },
  let_2i: { wgsl: 'let i = 2i; const r = V[i];', ok: false },
  let_3i: { wgsl: 'let i = 3i; const r = V[i];', ok: false },
  let_0u: { wgsl: 'let i = 0u; const r = V[i];', ok: false },
  let_1u: { wgsl: 'let i = 1u; const r = V[i];', ok: false },
  let_2u: { wgsl: 'let i = 2u; const r = V[i];', ok: false },
  let_3u: { wgsl: 'let i = 3u; const r = V[i];', ok: false },

  // indexing with 'var' variable
  var_0: { wgsl: 'var i = 0; const r = V[i];', ok: false },
  var_1: { wgsl: 'var i = 1; const r = V[i];', ok: false },
  var_2: { wgsl: 'var i = 2; const r = V[i];', ok: false },
  var_3: { wgsl: 'var i = 3; const r = V[i];', ok: false },
  var_0i: { wgsl: 'var i = 0i; const r = V[i];', ok: false },
  var_1i: { wgsl: 'var i = 1i; const r = V[i];', ok: false },
  var_2i: { wgsl: 'var i = 2i; const r = V[i];', ok: false },
  var_3i: { wgsl: 'var i = 3i; const r = V[i];', ok: false },
  var_0u: { wgsl: 'var i = 0u; const r = V[i];', ok: false },
  var_1u: { wgsl: 'var i = 1u; const r = V[i];', ok: false },
  var_2u: { wgsl: 'var i = 2u; const r = V[i];', ok: false },
  var_3u: { wgsl: 'var i = 3u; const r = V[i];', ok: false },

  // indexing with const expression
  const_expr_0: { wgsl: 'const r = V[0 / 2];', result_width: 1, ok: true },
  const_expr_1: { wgsl: 'const r = V[2 / 2];', result_width: 1, ok: true },
  const_expr_2: { wgsl: 'const r = V[4 / 2];', result_width: 1, ok: (width) => width > 2 },
  const_expr_3: { wgsl: 'const r = V[6 / 2];', result_width: 1, ok: (width) => width > 3 },
  const_expr_2_via_trig: {
    wgsl: 'const r = V[i32(tan(1.10714872) + 0.5)];',
    result_width: 1,
    ok: (width) => width > 2
  },
  const_expr_3_via_trig: {
    wgsl: 'const r = V[u32(tan(1.24904577) + 0.5)];',
    result_width: 1,
    ok: (width) => width > 3
  },
  const_expr_2_via_vec2: {
    wgsl: 'const r = V[vec2(3, 2)[1]];',
    result_width: 1,
    ok: (width) => width > 2
  },
  const_expr_3_via_vec2: {
    wgsl: 'const r = V[vec2(3, 2).x];',
    result_width: 1,
    ok: (width) => width > 3
  },
  const_expr_2_via_vec2u: {
    wgsl: 'const r = V[vec2u(3, 2)[1]];',
    result_width: 1,
    ok: (width) => width > 2
  },
  const_expr_3_via_vec2i: {
    wgsl: 'const r = V[vec2i(3, 2).x];',
    result_width: 1,
    ok: (width) => width > 3
  },
  const_expr_2_via_array: {
    wgsl: 'const r = V[array<i32, 2>(3, 2)[1]];',
    result_width: 1,
    ok: (width) => width > 2
  },
  const_expr_3_via_array: {
    wgsl: 'const r = V[array<i32, 2>(3, 2)[0]];',
    result_width: 1,
    ok: (width) => width > 3
  },
  const_expr_2_via_struct: {
    wgsl: 'const r = V[S(2).i];',
    result_width: 1,
    ok: (width) => width > 2
  },
  const_expr_3_via_struct: {
    wgsl: 'const r = V[S(3).i];',
    result_width: 1,
    ok: (width) => width > 3
  },

  // single element convenience name accesses
  x: { wgsl: 'const r = V.x;', result_width: 1, ok: true },
  y: { wgsl: 'const r = V.y;', result_width: 1, ok: true },
  z: { wgsl: 'const r = V.z;', result_width: 1, ok: (width) => width > 2 },
  w: { wgsl: 'const r = V.w;', result_width: 1, ok: (width) => width > 3 },
  r: { wgsl: 'const r = V.r;', result_width: 1, ok: true },
  g: { wgsl: 'const r = V.g;', result_width: 1, ok: true },
  b: { wgsl: 'const r = V.b;', result_width: 1, ok: (width) => width > 2 },
  a: { wgsl: 'const r = V.a;', result_width: 1, ok: (width) => width > 3 },

  // swizzles
  xy: { wgsl: 'const r = V.xy;', result_width: 2, ok: true },
  yx: { wgsl: 'const r = V.yx;', result_width: 2, ok: true },
  xyx: { wgsl: 'const r = V.xyx;', result_width: 3, ok: true },
  xyz: { wgsl: 'const r = V.xyz;', result_width: 3, ok: (width) => width > 2 },
  zyx: { wgsl: 'const r = V.zyx;', result_width: 3, ok: (width) => width > 2 },
  xyxy: { wgsl: 'const r = V.xyxy;', result_width: 4, ok: true },
  xyxz: { wgsl: 'const r = V.xyxz;', result_width: 4, ok: (width) => width > 2 },
  xyzw: { wgsl: 'const r = V.xyzw;', result_width: 4, ok: (width) => width > 3 },
  yxwz: { wgsl: 'const r = V.yxwz;', result_width: 4, ok: (width) => width > 3 },
  rg: { wgsl: 'const r = V.rg;', result_width: 2, ok: true },
  gr: { wgsl: 'const r = V.gr;', result_width: 2, ok: true },
  rgg: { wgsl: 'const r = V.rgg;', result_width: 3, ok: true },
  rgb: { wgsl: 'const r = V.rgb;', result_width: 3, ok: (width) => width > 2 },
  grb: { wgsl: 'const r = V.grb;', result_width: 3, ok: (width) => width > 2 },
  rgbr: { wgsl: 'const r = V.rgbr;', result_width: 4, ok: (width) => width > 2 },
  rgba: { wgsl: 'const r = V.rgba;', result_width: 4, ok: (width) => width > 3 },
  gbra: { wgsl: 'const r = V.gbra;', result_width: 4, ok: (width) => width > 3 },

  // swizzle chains
  xy_yx: { wgsl: 'const r = V.xy.yx;', result_width: 2, ok: true },
  xyx_xxy: { wgsl: 'const r = V.xyx.xxy;', result_width: 3, ok: true },
  xyz_zyx: { wgsl: 'const r = V.xyz.zyx;', result_width: 3, ok: (width) => width > 2 },
  xyxy_rrgg: { wgsl: 'const r = V.xyxy.rrgg;', result_width: 4, ok: true },
  rbrg_xyzw: { wgsl: 'const r = V.rbrg.xyzw;', result_width: 4, ok: (width) => width > 2 },
  xyxz_rbg_yx: {
    wgsl: 'const r = V.xyxz.rbg.yx;',
    result_width: 2,
    ok: (width) => width > 2
  },
  wxyz_bga_xy: {
    wgsl: 'const r = V.wxyz.bga.xy;',
    result_width: 2,
    ok: (width) => width > 3
  },

  // error: invalid convenience letterings
  xq: { wgsl: 'const r = V.xq;', ok: false },
  py: { wgsl: 'const r = V.py;', ok: false },

  // error: mixed convenience letterings
  xg: { wgsl: 'const r = V.xg;', ok: false },
  ryb: { wgsl: 'const r = V.ryb;', ok: false },
  xgza: { wgsl: 'const r = V.xgza;', ok: false },

  // error: too many swizzle elements
  xxxxx: { wgsl: 'const r = V.xxxxx;', ok: false },
  rrrrr: { wgsl: 'const r = V.rrrrr;', ok: false },
  yxwxy: { wgsl: 'const r = V.yxwxy;', ok: false },
  rgbar: { wgsl: 'const r = V.rgbar;', ok: false },

  // error: invalid index Value
  literal_5: { wgsl: 'const r = V[5];', ok: false },
  literal_minus_1: { wgsl: 'const r = V[-1];', ok: false },

  // error: invalid index type
  float_idx: { wgsl: 'const r = V[1.0];', ok: false },
  bool_idx: { wgsl: 'const r = V[true];', ok: false },
  array_idx: { wgsl: 'const r = V[array<i32, 2>()];', ok: false }
};

g.test('concrete').
desc('Tests validation of vector indexed and swizzles for concrete data types').
params((u) =>
u.
combine('vector_decl', ['const', 'let', 'var', 'param']).
combine('vector_width', [2, 3, 4]).
combine('element_type', ['i32', 'u32', 'f32', 'f16', 'bool']).
beginSubcases().
combine('case', keysOf(kConcreteCases))
).
beforeAllSubcases((t) => {
  if (t.params.element_type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const c = kConcreteCases[t.params.case];
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

g.test('abstract').
desc('Tests validation of vector indexed and swizzles for abstract data types').
params((u) =>
u.
combine('vector_width', [2, 3, 4]).
combine('abstract_type', ['int', 'float']).
combine('concrete_type', ['u32', 'i32', 'f32', 'f16']).
beginSubcases().
combine('case', keysOf(kAbstractCases))
).
beforeAllSubcases((t) => {
  if (t.params.concrete_type === 'f16') {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn((t) => {
  const enables = t.params.concrete_type === 'f16' ? 'enable f16;' : '';
  const c = kAbstractCases[t.params.case];
  const elem = t.params.abstract_type === 'int' ? '0' : '0.0';
  const vec_str = `vec${t.params.vector_width}(${Array(t.params.vector_width).
  fill(elem).
  join(', ')})`;

  const conversion_type =
  'result_width' in c ?
  c.result_width === 1 ?
  `${t.params.concrete_type}` :
  `vec${c.result_width}<${t.params.concrete_type}>` :
  '';
  const conversion = 'result_width' in c ? `const c: ${conversion_type} = r;` : '';

  const code = `${enables}
struct S {
  i : i32,
}

@compute @workgroup_size(1)
fn main() {
  ${c.wgsl.replace('V', vec_str)}
  ${conversion}
}
`;
  const convertible = isConvertible(
    Type[`abstract-${t.params.abstract_type}`],
    Type[t.params.concrete_type]
  );
  const pass = convertible && (typeof c.ok === 'function' ? c.ok(t.params.vector_width) : c.ok);
  t.expectCompileResult(pass, code);
});