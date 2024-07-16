/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Test memory layout requirements`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { keysOf } from '../../../common/util/data_tables.js';
import { iterRange } from '../../../common/util/util.js';
import { GPUTest } from '../../gpu_test.js';

export const g = makeTestGroup(GPUTest);












const kLayoutCases = {
  vec2u_align8: {
    type: `S_vec2u_align`,
    decl: `struct S_vec2u_align {
      x : u32,
      y : vec2u,
    }`,
    read_assign: `out = in.y[1]`,
    write_assign: `out.y[1] = in`,
    offset: 12
  },
  vec3u_align16: {
    type: `S_vec3u_align`,
    decl: `struct S_vec3u_align {
      x : u32,
      y : vec3u,
    }`,
    read_assign: `out = in.y[2]`,
    write_assign: `out.y[2] = in`,
    offset: 24
  },
  vec4u_align16: {
    type: `S_vec4u_align`,
    decl: `struct S_vec4u_align {
      x : u32,
      y : vec4u,
    }`,
    read_assign: `out = in.y[0]`,
    write_assign: `out.y[0] = in`,
    offset: 16
  },
  struct_align32: {
    type: `S_align32`,
    decl: `struct S_align32 {
      x : u32,
      @align(32) y : u32,
    }`,
    read_assign: `out = in.y;`,
    write_assign: `out.y = in`,
    offset: 32
  },
  vec2h_align4: {
    type: `S_vec2h_align`,
    decl: `struct S_vec2h_align {
      x : f16,
      y : vec2h,
    }`,
    read_assign: `out = u32(in.y[0])`,
    write_assign: `out.y[0] = f16(in)`,
    offset: 4,
    f16: true
  },
  vec3h_align8: {
    type: `S_vec3h_align`,
    decl: `struct S_vec3h_align {
      x : f16,
      y : vec3h,
    }`,
    read_assign: `out = u32(in.y[2])`,
    write_assign: `out.y[2] = f16(in)`,
    offset: 12,
    f16: true
  },
  vec4h_align8: {
    type: `S_vec4h_align`,
    decl: `struct S_vec4h_align {
      x : f16,
      y : vec4h,
    }`,
    read_assign: `out = u32(in.y[2])`,
    write_assign: `out.y[2] = f16(in)`,
    offset: 12,
    f16: true
  },
  vec2f_align8: {
    type: `S_vec2f_align`,
    decl: `struct S_vec2f_align {
      x : u32,
      y : vec2f,
    }`,
    read_assign: `out = u32(in.y[1])`,
    write_assign: `out.y[1] = f32(in)`,
    offset: 12,
    f32: true
  },
  vec3f_align16: {
    type: `S_vec3f_align`,
    decl: `struct S_vec3f_align {
      x : u32,
      y : vec3f,
    }`,
    read_assign: `out = u32(in.y[2])`,
    write_assign: `out.y[2] = f32(in)`,
    offset: 24,
    f32: true
  },
  vec4f_align16: {
    type: `S_vec4f_align`,
    decl: `struct S_vec4f_align {
      x : u32,
      y : vec4f,
    }`,
    read_assign: `out = u32(in.y[0])`,
    write_assign: `out.y[0] = f32(in)`,
    offset: 16,
    f32: true
  },
  vec3i_size12: {
    type: `S_vec3i_size`,
    decl: `struct S_vec3i_size {
      x : vec3i,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 12
  },
  vec3h_size6: {
    type: `S_vec3h_size`,
    decl: `struct S_vec3h_size {
      x : vec3h,
      y : f16,
      z : f16,
    }`,
    read_assign: `out = u32(in.z)`,
    write_assign: `out.z = f16(in)`,
    offset: 8,
    f16: true
  },
  size80: {
    type: `S_size80`,
    decl: `struct S_size80 {
      @size(80) x : u32,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 80
  },
  atomic_align4: {
    type: `S_atomic_align`,
    decl: `struct S_atomic_align {
      x : u32,
      y : atomic<u32>,
    }`,
    read_assign: `out = atomicLoad(&in.y)`,
    write_assign: `atomicStore(&out.y, in)`,
    offset: 4
  },
  atomic_size4: {
    type: `S_atomic_size`,
    decl: `struct S_atomic_size {
      x : atomic<u32>,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 4
  },
  mat2x2f_align8: {
    type: `S_mat2x2f_align`,
    decl: `struct S_mat2x2f_align {
      x : u32,
      y : mat2x2f,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f32(in)`,
    offset: 8,
    f32: true
  },
  mat3x2f_align8: {
    type: `S_mat3x2f_align`,
    decl: `struct S_mat3x2f_align {
      x : u32,
      y : mat3x2f,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f32(in)`,
    offset: 8,
    f32: true
  },
  mat4x2f_align8: {
    type: `S_mat4x2f_align`,
    decl: `struct S_mat4x2f_align {
      x : u32,
      y : mat4x2f,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f32(in)`,
    offset: 8,
    f32: true
  },
  mat2x3f_align16: {
    type: `S_mat2x3f_align`,
    decl: `struct S_mat2x3f_align {
      x : u32,
      y : mat2x3f,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f32(in)`,
    offset: 16,
    f32: true
  },
  mat3x3f_align16: {
    type: `S_mat3x3f_align`,
    decl: `struct S_mat3x3f_align {
      x : u32,
      y : mat3x3f,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f32(in)`,
    offset: 16,
    f32: true
  },
  mat4x3f_align16: {
    type: `S_mat4x3f_align`,
    decl: `struct S_mat4x3f_align {
      x : u32,
      y : mat4x3f,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f32(in)`,
    offset: 16,
    f32: true
  },
  mat2x4f_align16: {
    type: `S_mat2x4f_align`,
    decl: `struct S_mat2x4f_align {
      x : u32,
      y : mat2x4f,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f32(in)`,
    offset: 16,
    f32: true
  },
  mat3x4f_align16: {
    type: `S_mat3x4f_align`,
    decl: `struct S_mat3x4f_align {
      x : u32,
      y : mat3x4f,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f32(in)`,
    offset: 16,
    f32: true
  },
  mat4x4f_align16: {
    type: `S_mat4x4f_align`,
    decl: `struct S_mat4x4f_align {
      x : u32,
      y : mat4x4f,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f32(in)`,
    offset: 16,
    f32: true
  },
  mat2x2h_align4: {
    type: `S_mat2x2h_align`,
    decl: `struct S_mat2x2h_align {
      x : u32,
      y : mat2x2h,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f16(in)`,
    offset: 4,
    f16: true
  },
  mat3x2h_align4: {
    type: `S_mat3x2h_align`,
    decl: `struct S_mat3x2h_align {
      x : u32,
      y : mat3x2h,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f16(in)`,
    offset: 4,
    f16: true
  },
  mat4x2h_align4: {
    type: `S_mat4x2h_align`,
    decl: `struct S_mat4x2h_align {
      x : u32,
      y : mat4x2h,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f16(in)`,
    offset: 4,
    f16: true
  },
  mat2x3h_align8: {
    type: `S_mat2x3h_align`,
    decl: `struct S_mat2x3h_align {
      x : u32,
      y : mat2x3h,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f16(in)`,
    offset: 8,
    f16: true
  },
  mat3x3h_align8: {
    type: `S_mat3x3h_align`,
    decl: `struct S_mat3x3h_align {
      x : u32,
      y : mat2x3h,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f16(in)`,
    offset: 8,
    f16: true
  },
  mat4x3h_align8: {
    type: `S_mat4x3h_align`,
    decl: `struct S_mat4x3h_align {
      x : u32,
      y : mat4x3h,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f16(in)`,
    offset: 8,
    f16: true
  },
  mat2x4h_align8: {
    type: `S_mat2x4h_align`,
    decl: `struct S_mat2x4h_align {
      x : u32,
      y : mat2x4h,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f16(in)`,
    offset: 8,
    f16: true
  },
  mat3x4h_align8: {
    type: `S_mat3x4h_align`,
    decl: `struct S_mat3x4h_align {
      x : u32,
      y : mat3x4h,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f16(in)`,
    offset: 8,
    f16: true
  },
  mat4x4h_align8: {
    type: `S_mat4x4h_align`,
    decl: `struct S_mat4x4h_align {
      x : u32,
      y : mat4x4h,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f16(in)`,
    offset: 8,
    f16: true
  },
  mat2x2f_size: {
    type: `S_mat2x2f_size`,
    decl: `struct S_mat2x2f_size {
      x : mat2x2f,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 16
  },
  mat3x2f_size: {
    type: `S_mat3x2f_size`,
    decl: `struct S_mat3x2f_size {
      x : mat3x2f,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 24
  },
  mat4x2f_size: {
    type: `S_mat4x2f_size`,
    decl: `struct S_mat4x2f_size {
      x : mat4x2f,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 32
  },
  mat2x3f_size: {
    type: `S_mat2x3f_size`,
    decl: `struct S_mat2x3f_size {
      x : mat2x3f,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 32
  },
  mat3x3f_size: {
    type: `S_mat3x3f_size`,
    decl: `struct S_mat3x3f_size {
      x : mat3x3f,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 48
  },
  mat4x3f_size: {
    type: `S_mat4x3f_size`,
    decl: `struct S_mat4x3f_size {
      x : mat4x3f,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 64
  },
  mat2x4f_size: {
    type: `S_mat2x4f_size`,
    decl: `struct S_mat2x4f_size {
      x : mat2x4f,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 32
  },
  mat3x4f_size: {
    type: `S_mat3x4f_size`,
    decl: `struct S_mat3x4f_size {
      x : mat3x4f,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 48
  },
  mat4x4f_size: {
    type: `S_mat4x4f_size`,
    decl: `struct S_mat4x4f_size {
      x : mat4x4f,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 64
  },
  mat2x2h_size: {
    type: `S_mat2x2h_size`,
    decl: `struct S_mat2x2h_size {
      x : mat2x2h,
      y : f16,
    }`,
    read_assign: `out = u32(in.y)`,
    write_assign: `out.y = f16(in)`,
    offset: 8,
    f16: true
  },
  mat3x2h_size: {
    type: `S_mat3x2h_size`,
    decl: `struct S_mat3x2h_size {
      x : mat3x2h,
      y : f16,
    }`,
    read_assign: `out = u32(in.y)`,
    write_assign: `out.y = f16(in)`,
    offset: 12,
    f16: true
  },
  mat4x2h_size: {
    type: `S_mat4x2h_size`,
    decl: `struct S_mat4x2h_size {
      x : mat4x2h,
      y : f16,
    }`,
    read_assign: `out = u32(in.y)`,
    write_assign: `out.y = f16(in)`,
    offset: 16,
    f16: true
  },
  mat2x3h_size: {
    type: `S_mat2x3h_size`,
    decl: `struct S_mat2x3h_size {
      x : mat2x3h,
      y : f16,
    }`,
    read_assign: `out = u32(in.y)`,
    write_assign: `out.y = f16(in)`,
    offset: 16,
    f16: true
  },
  mat3x3h_size: {
    type: `S_mat3x3h_size`,
    decl: `struct S_mat3x3h_size {
      x : mat3x3h,
      y : f16,
    }`,
    read_assign: `out = u32(in.y)`,
    write_assign: `out.y = f16(in)`,
    offset: 24,
    f16: true
  },
  mat4x3h_size: {
    type: `S_mat4x3h_size`,
    decl: `struct S_mat4x3h_size {
      x : mat4x3h,
      y : f16,
    }`,
    read_assign: `out = u32(in.y)`,
    write_assign: `out.y = f16(in)`,
    offset: 32,
    f16: true
  },
  mat2x4h_size: {
    type: `S_mat2x4h_size`,
    decl: `struct S_mat2x4h_size {
      x : mat2x4h,
      y : f16,
    }`,
    read_assign: `out = u32(in.y)`,
    write_assign: `out.y = f16(in)`,
    offset: 16,
    f16: true
  },
  mat3x4h_size: {
    type: `S_mat3x4h_size`,
    decl: `struct S_mat3x4h_size {
      x : mat3x4h,
      y : f16,
    }`,
    read_assign: `out = u32(in.y)`,
    write_assign: `out.y = f16(in)`,
    offset: 24,
    f16: true
  },
  mat4x4h_size: {
    type: `S_mat4x4h_size`,
    decl: `struct S_mat4x4h_size {
      x : mat4x4h,
      y : f16,
    }`,
    read_assign: `out = u32(in.y)`,
    write_assign: `out.y = f16(in)`,
    offset: 32,
    f16: true
  },
  struct_align_vec2i: {
    type: `S_struct_align_vec2i`,
    decl: `struct Inner {
      x : u32,
      y : vec2i,
    }
    struct S_struct_align_vec2i {
      x : u32,
      y : Inner,
    }`,
    read_assign: `out = in.y.x`,
    write_assign: `out.y.x = in`,
    offset: 8,
    skip_uniform: true
  },
  struct_align_vec3i: {
    type: `S_struct_align_vec3i`,
    decl: `struct Inner {
      x : u32,
      y : vec3i,
    }
    struct S_struct_align_vec3i {
      x : u32,
      y : Inner,
    }`,
    read_assign: `out = in.y.x`,
    write_assign: `out.y.x = in`,
    offset: 16
  },
  struct_align_vec4i: {
    type: `S_struct_align_vec4i`,
    decl: `struct Inner {
      x : u32,
      y : vec4i,
    }
    struct S_struct_align_vec4i {
      x : u32,
      y : Inner,
    }`,
    read_assign: `out = in.y.x`,
    write_assign: `out.y.x = in`,
    offset: 16
  },
  struct_align_vec2h: {
    type: `S_struct_align_vec2h`,
    decl: `struct Inner {
      x : f16,
      y : vec2h,
    }
    struct S_struct_align_vec2h {
      x : f16,
      y : Inner,
    }`,
    read_assign: `out = u32(in.y.x)`,
    write_assign: `out.y.x = f16(in)`,
    offset: 4,
    f16: true,
    skip_uniform: true
  },
  struct_align_vec3h: {
    type: `S_struct_align_vec3h`,
    decl: `struct Inner {
      x : f16,
      y : vec3h,
    }
    struct S_struct_align_vec3h {
      x : f16,
      y : Inner,
    }`,
    read_assign: `out = u32(in.y.x)`,
    write_assign: `out.y.x = f16(in)`,
    offset: 8,
    f16: true,
    skip_uniform: true
  },
  struct_align_vec4h: {
    type: `S_struct_align_vec4h`,
    decl: `struct Inner {
      x : f16,
      y : vec4h,
    }
    struct S_struct_align_vec4h {
      x : f16,
      y : Inner,
    }`,
    read_assign: `out = u32(in.y.x)`,
    write_assign: `out.y.x = f16(in)`,
    offset: 8,
    f16: true,
    skip_uniform: true
  },
  struct_size_roundup: {
    type: `S_struct_size_roundup`,
    decl: `struct Inner {
      x : vec3u,
    }
    struct S_struct_size_roundup {
      x : Inner,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 16
  },
  struct_inner_size: {
    type: `S_struct_inner_size`,
    decl: `struct Inner {
      @size(112) x : u32,
    }
    struct S_struct_inner_size {
      x : Inner,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 112
  },
  struct_inner_align: {
    type: `S_struct_inner_align`,
    decl: `struct Inner {
      @align(64) x : u32,
    }
    struct S_struct_inner_align {
      x : u32,
      y : Inner,
    }`,
    read_assign: `out = in.y.x`,
    write_assign: `out.y.x = in`,
    offset: 64
  },
  struct_inner_size_and_align: {
    type: `S_struct_inner_size_and_align`,
    decl: `struct Inner {
      @align(32) @size(33) x : u32,
    }
    struct S_struct_inner_size_and_align {
      x : Inner,
      y : Inner,
    }`,
    read_assign: `out = in.y.x`,
    write_assign: `out.y.x = in`,
    offset: 64
  },
  struct_override_size: {
    type: `S_struct_override_size`,
    decl: `struct Inner {
      @size(32) x : u32,
    }
    struct S_struct_override_size {
      @size(64) x : Inner,
      y : u32,
    }`,
    read_assign: `out = in.y`,
    write_assign: `out.y = in`,
    offset: 64
  },
  struct_double_align: {
    type: `S_struct_double_align`,
    decl: `struct Inner {
      x : u32,
      @align(32) y : u32,
    }
    struct S_struct_double_align {
      x : u32,
      @align(64) y : Inner,
    }`,
    read_assign: `out = in.y.y`,
    write_assign: `out.y.y = in`,
    offset: 96
  },
  array_vec3u_align: {
    type: `S_array_vec3u_align`,
    decl: `struct S_array_vec3u_align {
      x : u32,
      y : array<vec3u, 2>,
    }`,
    read_assign: `out = in.y[0][0]`,
    write_assign: `out.y[0][0] = in`,
    offset: 16
  },
  array_vec3h_align: {
    type: `S_array_vec3h_align`,
    decl: `struct S_array_vec3h_align {
      x : f16,
      y : array<vec3h, 2>,
    }`,
    read_assign: `out = u32(in.y[0][0])`,
    write_assign: `out.y[0][0] = f16(in)`,
    offset: 8,
    f16: true,
    skip_uniform: true
  },
  array_vec3u_stride: {
    type: `S_array_vec3u_stride`,
    decl: `struct S_array_vec3u_stride {
      x : array<vec3u, 4>,
    }`,
    read_assign: `out = in.x[1][0]`,
    write_assign: `out.x[1][0] = in`,
    offset: 16
  },
  array_vec3h_stride: {
    type: `S_array_vec3h_stride`,
    decl: `struct S_array_vec3h_stride {
      x : array<vec3h, 4>,
    }`,
    read_assign: `out = u32(in.x[1][0])`,
    write_assign: `out.x[1][0] = f16(in)`,
    offset: 8,
    f16: true,
    skip_uniform: true
  },
  array_stride_size: {
    type: `array<S_stride, 4>`,
    decl: `struct S_stride {
      @size(16) x : u32,
    }`,
    read_assign: `out = in[2].x`,
    write_assign: `out[2].x = in`,
    offset: 32
  },
  array_mat2x2f_stride: {
    type: `array<mat2x2f, 4>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f32(in)`,
    offset: 16,
    f32: true
  },
  array_mat2x2h_stride: {
    type: `array<mat2x2h, 2>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f16(in)`,
    offset: 8,
    f16: true,
    skip_uniform: true
  },
  array_mat3x2f_stride: {
    type: `array<mat3x2f, 3>`,
    read_assign: `out = u32(in[2][0][0])`,
    write_assign: `out[2][0][0] = f32(in)`,
    offset: 48,
    f32: true,
    skip_uniform: true
  },
  array_mat3x2h_stride: {
    type: `array<mat3x2h, 2>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f16(in)`,
    offset: 12,
    f16: true,
    skip_uniform: true
  },
  array_mat4x2f_stride: {
    type: `array<mat4x2f, 4>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f32(in)`,
    offset: 32,
    f32: true
  },
  array_mat4x2h_stride: {
    type: `array<mat4x2h, 2>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f16(in)`,
    offset: 16,
    f16: true
  },
  array_mat2x3f_stride: {
    type: `array<mat2x3f, 4>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f32(in)`,
    offset: 32,
    f32: true
  },
  array_mat2x3h_stride: {
    type: `array<mat2x3h, 2>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f16(in)`,
    offset: 16,
    f16: true
  },
  array_mat3x3f_stride: {
    type: `array<mat3x3f, 3>`,
    read_assign: `out = u32(in[2][0][0])`,
    write_assign: `out[2][0][0] = f32(in)`,
    offset: 96,
    f32: true
  },
  array_mat3x3h_stride: {
    type: `array<mat3x3h, 2>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f16(in)`,
    offset: 24,
    f16: true,
    skip_uniform: true
  },
  array_mat4x3f_stride: {
    type: `array<mat4x3f, 4>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f32(in)`,
    offset: 64,
    f32: true
  },
  array_mat4x3h_stride: {
    type: `array<mat4x3h, 2>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f16(in)`,
    offset: 32,
    f16: true
  },
  array_mat2x4f_stride: {
    type: `array<mat2x4f, 4>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f32(in)`,
    offset: 32,
    f32: true
  },
  array_mat2x4h_stride: {
    type: `array<mat2x4h, 2>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f16(in)`,
    offset: 16,
    f16: true
  },
  array_mat3x4f_stride: {
    type: `array<mat3x4f, 3>`,
    read_assign: `out = u32(in[2][0][0])`,
    write_assign: `out[2][0][0] = f32(in)`,
    offset: 96,
    f32: true
  },
  array_mat3x4h_stride: {
    type: `array<mat3x4h, 2>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f16(in)`,
    offset: 24,
    f16: true,
    skip_uniform: true
  },
  array_mat4x4f_stride: {
    type: `array<mat4x4f, 4>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f32(in)`,
    offset: 64,
    f32: true
  },
  array_mat4x4h_stride: {
    type: `array<mat4x4h, 2>`,
    read_assign: `out = u32(in[1][0][0])`,
    write_assign: `out[1][0][0] = f16(in)`,
    offset: 32,
    f16: true
  }
};

g.test('read_layout').
desc('Test reading memory layouts').
params((u) =>
u.
combine('case', keysOf(kLayoutCases)).
combine('aspace', ['storage', 'uniform', 'workgroup', 'function', 'private']).
beginSubcases()
).
beforeAllSubcases((t) => {
  const testcase = kLayoutCases[t.params.case];
  if (testcase.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
  // Don't test atomics in workgroup due to initialization boilerplate.
  t.skipIf(
    testcase.type.includes('atomic') && t.params.aspace !== 'storage',
    `Skipping atomic test for non-storage address space`
  );

  t.skipIf(
    testcase.skip_uniform === true && t.params.aspace === 'uniform',
    `Uniform requires 16 byte alignment`
  );
}).
fn((t) => {
  const testcase = kLayoutCases[t.params.case];
  let code = `
${testcase.f16 ? 'enable f16;' : ''}
${testcase.decl ?? ''}

@group(0) @binding(1)
var<storage, read_write> out : u32;
`;

  if (t.params.aspace === 'uniform') {
    code += `@group(0) @binding(0)
      var<${t.params.aspace}> in : ${testcase.type};`;
  } else if (t.params.aspace === 'storage') {
    // Use read_write for input data to support atomics.
    code += `@group(0) @binding(0)
      var<${t.params.aspace}, read_write> in : ${testcase.type};`;
  } else {
    code += `@group(0) @binding(0)
      var<storage> pre_in : ${testcase.type};`;
    if (t.params.aspace === 'workgroup') {
      code += `
        var<workgroup> in : ${testcase.type};`;
    } else if (t.params.aspace === 'private') {
      code += `
        var<private> in : ${testcase.type};`;
    }
  }

  code += `
@compute @workgroup_size(1,1,1)
fn main() {
`;

  if (
  t.params.aspace === 'workgroup' ||
  t.params.aspace === 'function' ||
  t.params.aspace === 'private')
  {
    if (t.params.aspace === 'function') {
      code += `var in : ${testcase.type};\n`;
    }
    code += `in = pre_in;`;
    if (t.params.aspace === 'workgroup') {
      code += `workgroupBarrier();\n`;
    }
  }

  code += `\n${testcase.read_assign};\n}`;

  let usage = GPUBufferUsage.COPY_SRC;
  if (t.params.aspace === 'uniform') {
    usage |= GPUBufferUsage.UNIFORM;
  } else {
    usage |= GPUBufferUsage.STORAGE;
  }

  // Magic number is 42 in various representations.
  const inMagicNumber = testcase.f16 ? 0x5140 : testcase.f32 ? 0x42280000 : 42;
  const in_buffer = t.makeBufferWithContents(
    new Uint32Array([
    ...iterRange(128, (x) => {
      if (x * 4 === testcase.offset) {
        return inMagicNumber;
      } else {
        return 0;
      }
    })]
    ),
    usage
  );

  const out_buffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(1, (x) => 0)]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code
      }),
      entryPoint: 'main'
    }
  });

  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer: in_buffer
      }
    },
    {
      binding: 1,
      resource: {
        buffer: out_buffer
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(1, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(out_buffer, new Uint32Array([42]));
});

g.test('write_layout').
desc('Test writing memory layouts').
params((u) =>
u.
combine('case', keysOf(kLayoutCases)).
combine('aspace', ['storage', 'workgroup', 'function', 'private']).
beginSubcases()
).
beforeAllSubcases((t) => {
  const testcase = kLayoutCases[t.params.case];
  if (testcase.f16) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
  // Don't test atomics in workgroup due to initialization boilerplate.
  t.skipIf(
    testcase.type.includes('atomic') && t.params.aspace !== 'storage',
    `Skipping atomic test for non-storage address space`
  );
}).
fn((t) => {
  const testcase = kLayoutCases[t.params.case];
  let code = `
${testcase.f16 ? 'enable f16;' : ''}
${testcase.decl ?? ''}

@group(0) @binding(0)
var<storage> in : u32;
`;

  if (t.params.aspace === 'storage') {
    code += `@group(0) @binding(1)
      var<storage, read_write> out : ${testcase.type};\n`;
  } else {
    code += `@group(0) @binding(1)
      var<storage, read_write> post_out : ${testcase.type};\n`;

    if (t.params.aspace === 'workgroup') {
      code += `var<workgroup> out : ${testcase.type};\n`;
    } else if (t.params.aspace === 'private') {
      code += `var<private> out : ${testcase.type};\n`;
    }
  }

  code += `
@compute @workgroup_size(1,1,1)
fn main() {
`;

  if (t.params.aspace === 'function') {
    code += `var out : ${testcase.type};\n`;
  }

  code += `${testcase.write_assign};\n`;
  if (
  t.params.aspace === 'workgroup' ||
  t.params.aspace === 'function' ||
  t.params.aspace === 'private')
  {
    if (t.params.aspace === 'workgroup') {
      code += `workgroupBarrier();\n`;
    }
    code += `post_out = out;`;
  }

  code += `\n}`;

  const in_buffer = t.makeBufferWithContents(
    new Uint32Array([42]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  );

  const out_buffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(128, (x) => 0)]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code
      }),
      entryPoint: 'main'
    }
  });

  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer: in_buffer
      }
    },
    {
      binding: 1,
      resource: {
        buffer: out_buffer
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(1, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  // Magic number is 42 in various representations.
  const outMagicNumber = testcase.f16 ? 0x5140 : testcase.f32 ? 0x42280000 : 42;
  const expect = new Uint32Array([
  ...iterRange(128, (x) => {
    if (x * 4 === testcase.offset) {
      return outMagicNumber;
    } else {
      return 0;
    }
  })]
  );

  t.expectGPUBufferValuesEqual(out_buffer, expect);
});