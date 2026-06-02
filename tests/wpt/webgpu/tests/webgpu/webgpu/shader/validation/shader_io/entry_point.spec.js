/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for attributes and entry point requirements`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('missing_attribute_on_param').
desc(`Test that an entry point without an IO attribute on one of its parameters is rejected.`).
params((u) =>
u.combine('target_stage', ['', 'vertex', 'fragment', 'compute']).beginSubcases()
).
fn((t) => {
  const vertex_attr = t.params.target_stage === 'vertex' ? '' : '@location(1)';
  const fragment_attr = t.params.target_stage === 'fragment' ? '' : '@location(1)';
  const compute_attr = t.params.target_stage === 'compute' ? '' : '@builtin(workgroup_id)';
  const code = `
@vertex
fn vert_main(@location(0) a : f32,
             ${vertex_attr}  b : f32,
@             location(2) c : f32) -> @builtin(position) vec4<f32> {
  return vec4<f32>();
}

@fragment
fn frag_main(@location(0)  a : f32,
             ${fragment_attr} b : f32,
@             location(2)  c : f32) {
}

@compute @workgroup_size(1)
fn comp_main(@builtin(global_invocation_id) a : vec3<u32>,
             ${compute_attr}                   b : vec3<u32>,
             @builtin(local_invocation_id)  c : vec3<u32>) {
}
`;
  t.expectCompileResult(t.params.target_stage === '', code);
});

g.test('missing_attribute_on_param_struct').
desc(
  `Test that an entry point struct parameter without an IO attribute on one of its members is rejected.`
).
params((u) =>
u.combine('target_stage', ['', 'vertex', 'fragment', 'compute']).beginSubcases()
).
fn((t) => {
  const vertex_attr = t.params.target_stage === 'vertex' ? '' : '@location(1)';
  const fragment_attr = t.params.target_stage === 'fragment' ? '' : '@location(1)';
  const compute_attr = t.params.target_stage === 'compute' ? '' : '@builtin(workgroup_id)';
  const code = `
struct VertexInputs {
  @location(0) a : f32,
  ${vertex_attr}  b : f32,
@  location(2) c : f32,
};
struct FragmentInputs {
  @location(0)  a : f32,
  ${fragment_attr} b : f32,
@  location(2)  c : f32,
};
struct ComputeInputs {
  @builtin(global_invocation_id) a : vec3<u32>,
  ${compute_attr}                   b : vec3<u32>,
  @builtin(local_invocation_id)  c : vec3<u32>,
};

@vertex
fn vert_main(inputs : VertexInputs) -> @builtin(position) vec4<f32> {
  return vec4<f32>();
}

@fragment
fn frag_main(inputs : FragmentInputs) {
}

@compute @workgroup_size(1)
fn comp_main(inputs : ComputeInputs) {
}
`;
  t.expectCompileResult(t.params.target_stage === '', code);
});

g.test('missing_attribute_on_return_type').
desc(`Test that an entry point without an IO attribute on its return type is rejected.`).
params((u) => u.combine('target_stage', ['', 'vertex', 'fragment']).beginSubcases()).
fn((t) => {
  const vertex_attr = t.params.target_stage === 'vertex' ? '' : '@builtin(position)';
  const fragment_attr = t.params.target_stage === 'fragment' ? '' : '@location(0)';
  const code = `
@vertex
fn vert_main() -> ${vertex_attr} vec4<f32> {
  return vec4<f32>();
}

@fragment
fn frag_main() -> ${fragment_attr} vec4<f32> {
  return vec4<f32>();
}
`;
  t.expectCompileResult(t.params.target_stage === '', code);
});

g.test('missing_attribute_on_return_type_struct').
desc(
  `Test that an entry point struct return type without an IO attribute on one of its members is rejected.`
).
params((u) => u.combine('target_stage', ['', 'vertex', 'fragment']).beginSubcases()).
fn((t) => {
  const vertex_attr = t.params.target_stage === 'vertex' ? '' : '@location(1)';
  const fragment_attr = t.params.target_stage === 'fragment' ? '' : '@location(1)';
  const code = `
struct VertexOutputs {
  @location(0)       a : f32,
  ${vertex_attr}        b : f32,
  @builtin(position) c : vec4<f32>,
};
struct FragmentOutputs {
  @location(0)  a : f32,
  ${fragment_attr} b : f32,
@  location(2)  c : f32,
};

@vertex
fn vert_main() -> VertexOutputs {
  return VertexOutputs();
}

@fragment
fn frag_main() -> FragmentOutputs {
  return FragmentOutputs();
}
`;
  t.expectCompileResult(t.params.target_stage === '', code);
});

g.test('no_entry_point_provided').
desc(`Tests that a shader without an entry point is accepted`).
fn((t) => {
  t.expectCompileResult(true, 'fn main() {}');
});