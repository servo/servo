/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validate immediate data usage in RenderPassEncoder, ComputePassEncoder, and RenderBundleEncoder.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { getGPU } from '../../../../../common/util/navigator_gpu.js';
import { supportsImmediateData, unreachable } from '../../../../../common/util/util.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../gpu_test.js';
import { kProgrammableEncoderTypes } from '../../../../util/command_buffer_maker.js';

class PipelineImmediateTest extends AllFeaturesMaxLimitsGPUTest {
  async init() {
    await super.init();
    if (!supportsImmediateData(getGPU(this.rec))) {
      this.skip('setImmediates not supported');
    }
  }

  runPass(
  encoder,
  code,
  immediateSize)
  {
    const layout = this.device.createPipelineLayout({
      bindGroupLayouts: [],
      immediateSize
    });

    if (encoder instanceof GPUComputePassEncoder) {
      const pipeline = this.device.createComputePipeline({
        layout,
        compute: {
          module: this.device.createShaderModule({ code })
        }
      });
      encoder.setPipeline(pipeline);
      encoder.dispatchWorkgroups(1);
    } else {
      const pipeline = this.device.createRenderPipeline({
        layout,
        vertex: {
          module: this.device.createShaderModule({ code })
        },
        fragment: {
          module: this.device.createShaderModule({ code }),
          targets: [{ format: 'rgba8unorm' }]
        }
      });
      encoder.setPipeline(pipeline);
      encoder.draw(3);
    }
  }
}

export const g = makeTestGroup(PipelineImmediateTest);

g.test('required_slots_set').
desc(
  `
    Validate that all immediate data slots required by the pipeline are set on the encoder.
    - For each immediate data variable statically used by the pipeline:
      - All accessible slots (4-byte words) must be set via setImmediates.
    - Scenarios:
      - scalar: Simple u32 usage.
      - vector: Simple vec4<u32> usage.
      - struct_padding: Struct with padding. Padding bytes do not need to be set.
        When a struct variable is statically used, all non-padding bytes of the entire struct must be set,
        even if only one member is accessed. In this test, data.b.v is accessed, but both data.a and data.b
        must be set (excluding padding).
      - dynamic_indexing: Array with dynamic indexing.
    - Usage:
      - full: Set all declared bytes.
      - split: Set all declared bytes in multiple calls.
      - partial: Set only a subset of bytes.
        - For struct_padding, this means setting only members (no padding), which is valid.
        - For others, this means missing required data, which is invalid.
      - overprovision: Set more bytes than declared, but in range of layout immediateSize.
    `
).
params((u) =>
u.
combine('encoderType', kProgrammableEncoderTypes).
combine('scenario', [
'scalar',
'vector',
'struct_padding',
'dynamic_indexing',
'mixed_types',
'multiple_variables']
).
combine('usage', ['full', 'partial', 'split', 'overprovision']).
expand('stage', (p) => {
  if (p.encoderType === 'compute pass') return ['compute'];
  return ['vertex', 'fragment', 'both'];
}).
unless((p) => p.scenario === 'scalar' && p.usage === 'split')
).
fn((t) => {
  const { encoderType, scenario, usage, stage } = t.params;

  let code = '';
  let layoutImmediateSize = 0;
  let trailingPaddingBytes = 0;

  const use_vertex = stage === 'vertex' || stage === 'both';
  const use_fragment = stage === 'fragment' || stage === 'both';
  const both_different = stage === 'both';

  let declarations = '';
  let helpers = '';
  let callCompute = 'use_data();';
  let callVertex = 'use_data();';
  let callFragment = 'use_data();';
  let computeArgs = '';
  let vertexArgs = '';
  let fragmentArgs = '';
  let fragmentPrelude = '';

  switch (scenario) {
    case 'scalar':
      layoutImmediateSize = 4;
      declarations = 'var<immediate> data: u32;';
      helpers = 'fn use_data() { _ = data; }';
      break;
    case 'vector':
      layoutImmediateSize = 16;
      declarations = 'var<immediate> data: vec4<u32>;';
      helpers = 'fn use_data() { _ = data; }';
      break;
    case 'struct_padding':
      layoutImmediateSize = 64;
      trailingPaddingBytes = 24;
      declarations = `
          struct A { v: vec3<u32> }
          struct B { v: vec2<u32> }
          struct Data { a: A, @align(32) b: B, }
          var<immediate> data: Data;`;
      helpers = `
          fn use_a() { _ = data.a.v; }
          fn use_b() { _ = data.b.v; }`;
      callCompute = 'use_b();';
      callVertex = both_different ? 'use_a();' : 'use_b();';
      callFragment = 'use_b();';
      break;
    case 'mixed_types':
      layoutImmediateSize = 32;
      declarations = `
          struct Mixed { v: u32, f: vec4<u32> }
          var<immediate> data: Mixed;`;
      helpers = `
          fn use_v() { _ = data.v; }
          fn use_f() { _ = data.f; }`;
      callCompute = 'use_f();';
      callVertex = both_different ? 'use_v();' : 'use_f();';
      callFragment = 'use_f();';
      break;
    case 'dynamic_indexing':
      layoutImmediateSize = 16;
      declarations = 'var<immediate> data: array<u32, 4>;';
      helpers = 'fn use_data(i: u32) { _ = data[i]; }';
      computeArgs = '@builtin(local_invocation_index) i: u32';
      callCompute = 'use_data(i);';
      vertexArgs = '@builtin(vertex_index) i: u32';
      callVertex = 'use_data(i);';
      fragmentArgs = '@builtin(position) pos: vec4<f32>';
      fragmentPrelude = 'let i = u32(pos.x);';
      callFragment = 'use_data(i);';
      break;
    case 'multiple_variables':
      layoutImmediateSize = 32;
      declarations = `
          struct S1 { a: u32, x: u32 }
          struct S2 { a: u32, y: vec4<u32> }
          var<immediate> v1: S1;
          var<immediate> v2: S2;`;
      helpers = `
          fn use_v1() { _ = v1.a; }
          fn use_v2() { _ = v2.a; }`;
      callCompute = 'use_v2();';
      callVertex = both_different ? 'use_v1();' : 'use_v2();';
      callFragment = 'use_v2();';
      break;
  }

  code = `
      ${declarations}
      ${helpers}
      @compute @workgroup_size(1) fn main_compute(${computeArgs}) {
        ${callCompute}
      }
      @vertex fn main_vertex(${vertexArgs}) -> @builtin(position) vec4<f32> {
        ${use_vertex ? callVertex : ''}
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
      }
      @fragment fn main_fragment(${fragmentArgs}) -> @location(0) vec4<f32> {
        ${fragmentPrelude}
        ${use_fragment ? callFragment : ''}
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
      }
    `;

  const kRequiredSize = layoutImmediateSize - trailingPaddingBytes;
  // When overprovisioning: if the struct already has trailing padding, the layout
  // size is already larger than what the shader uses, so no extra bytes are needed.
  // Only when there's no trailing padding do we need to increase the layout size.
  const layoutSize =
  usage === 'overprovision' && trailingPaddingBytes === 0 ?
  layoutImmediateSize + 4 :
  layoutImmediateSize;
  if (layoutSize > t.device.limits.maxImmediateSize) {
    t.skip('maxImmediateSize not large enough for overprovision test');
  }

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType);

  const setImmediates = (offset, size) => {
    const data = new Uint8Array(size);
    encoder.setImmediates(offset, data, 0, size);
  };

  if (usage === 'overprovision') {
    // When trailingPaddingBytes > 0, the layout already overprovisions beyond
    // shader usage, so set the full layoutImmediateSize. When trailingPaddingBytes
    // is 0, we added +4 to the layout, so set kRequiredSize + 4.
    setImmediates(0, trailingPaddingBytes > 0 ? layoutImmediateSize : kRequiredSize + 4);
  } else if (usage === 'full') {
    setImmediates(0, kRequiredSize);
  } else if (usage === 'partial') {
    if (scenario === 'multiple_variables') {
      if (stage === 'both') {
        // Test missing required bytes in vertex but padding bytes in fragment.
        // struct S1 { a: u32, x: u32 }       // slots used: 1100 0000
        // struct S2 { a: u32, y: vec4<u32> } // slots used: 1000 1111
        // total slots used:                                 1100 1111
        // Do upload slots:                                  1000 1111
        setImmediates(0, 4);
        setImmediates(16, 16);
      } else if (stage === 'vertex') {
        // Missing required bytes in vertex.
        // struct S1 { a: u32, x: u32 }       // slots used: 1100 0000
        // Do upload slots:                   //             1000 0000
        setImmediates(0, 4);
      } else if (stage === 'fragment' || stage === 'compute') {
        // Missing required bytes in fragment and compute
        // struct S2 { a: u32, y: vec4<u32> } // slots used: 1000 1111
        // Do upload slots:                   //             0000 1111
        setImmediates(16, 16);
      } else {
        unreachable();
      }
    } else {
      const partialSize = kRequiredSize >= 8 ? kRequiredSize / 2 : 0;
      setImmediates(0, partialSize);
    }
  } else if (usage === 'split') {
    if (scenario === 'struct_padding') {
      // struct Data { a: A, @align(32) b: B, } slots used : // 1110 1111
      setImmediates(0, 12);
      setImmediates(32, 8);
    } else if (scenario === 'mixed_types') {
      // struct Mixed { v: u32, f: vec4<u32> } slots used : // 1000 1111
      setImmediates(0, 4);
      setImmediates(16, 16);
    } else if (scenario === 'multiple_variables') {
      // struct S1 { a: u32, x: u32 }       // slots used: 1100 0000
      // struct S2 { a: u32, y: vec4<u32> } // slots used: 1000 1111
      // total slots used:                                 1100 1111
      setImmediates(0, 8);
      setImmediates(16, 16);
    } else if (scenario === 'vector' || scenario === 'dynamic_indexing') {
      // Vector (size 16), dynamic_indexing (size 16)
      setImmediates(0, 8);
      setImmediates(8, 8);
    } else {
      unreachable();
    }
  } else {
    unreachable();
  }

  t.runPass(encoder, code, layoutSize);

  const shouldSucceed = usage === 'full' || usage === 'split' || usage === 'overprovision';
  validateFinishAndSubmit(shouldSucceed, true);
});

g.test('unused_variable').
desc(
  `
    Validate that if an immediate data variable is declared but not statically used,
    it does not require slots to be set.
    `
).
params((u) =>
u.
combine('encoderType', kProgrammableEncoderTypes).
combine('usage', ['none', 'partial_start']).
combine('scenario', ['not_referenced', 'referenced_in_unused_function'])
).
fn((t) => {
  const { encoderType, usage, scenario } = t.params;
  const kImmediateSize = 16;

  const code =
  scenario === 'not_referenced' ?
  `
      var<immediate> data: vec4<u32>;

      @compute @workgroup_size(1) fn main_compute() {
        // data is not used
      }

      @vertex fn main_vertex() -> @builtin(position) vec4<f32> {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
      }

      @fragment fn main_fragment() -> @location(0) vec4<f32> {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
      }
    ` :
  // referenced_in_unused_function
  `
      var<immediate> data: vec4<u32>;
      fn unused_helper() { _ = data; }

      @compute @workgroup_size(1) fn main_compute() {
        // unused_helper is not called
      }

      @vertex fn main_vertex() -> @builtin(position) vec4<f32> {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
      }

      @fragment fn main_fragment() -> @location(0) vec4<f32> {
        return vec4<f32>(0.0, 1.0, 0.0, 1.0);
      }
    `;

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType);

  if (usage === 'partial_start') {
    const data = new Uint8Array(8);
    encoder.setImmediates(0, data, 0, 8);
  }

  t.runPass(encoder, code, kImmediateSize);

  validateFinishAndSubmit(true, true);
});

g.test('overprovisioned_immediate_data').
desc(
  `
    Validate that setting more immediate data than used by the shader is valid.
    - larger_than_shader: Set size matches layout size, but is larger than what shader uses.
    - larger_than_layout: Set size is larger than layout size (and shader usage).
      Validation only checks against maxImmediateSize, not pipeline layout immediateSize.
    `
).
params((u) =>
u.
combine('encoderType', kProgrammableEncoderTypes).
combine('scenario', ['larger_than_shader', 'larger_than_layout'])
).
fn((t) => {
  const { encoderType, scenario } = t.params;
  const kLayoutSize = 16;
  // Shader only uses 4 bytes (u32)
  const code = `
      var<immediate> data: u32;
      fn use_data() { _ = data; }
      @compute @workgroup_size(1) fn main_compute() { use_data(); }
      @vertex fn main_vertex() -> @builtin(position) vec4<f32> { use_data(); return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
      @fragment fn main_fragment() -> @location(0) vec4<f32> { use_data(); return vec4<f32>(0.0, 1.0, 0.0, 1.0); }
    `;

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType);

  const kSetSize = scenario === 'larger_than_layout' ? kLayoutSize + 4 : kLayoutSize;

  const data = new Uint8Array(kSetSize);
  encoder.setImmediates(0, data, 0, kSetSize);

  t.runPass(encoder, code, kLayoutSize);

  validateFinishAndSubmit(true, true);
});

g.test('render_bundle_execution_state_invalidation').
desc(
  `
    Validate that executeBundles invalidates the current immediate data state
    in the RenderPassEncoder.
    - Immediate data must be re-set after executeBundles.
    - setImmediates in bundle does not leak to pass.
    `
).
params((u) => u.beginSubcases().combine('resetImmediates', [true, false])).
fn((t) => {
  const { resetImmediates } = t.params;
  const kImmediateSize = 16;

  // Create a pipeline requiring immediate data
  const code = `
      var<immediate> data: vec4<u32>;
      fn use_data() { _ = data; }
      @vertex fn main_vertex() -> @builtin(position) vec4<f32> { use_data(); return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
      @fragment fn main_fragment() -> @location(0) vec4<f32> { use_data(); return vec4<f32>(0.0, 1.0, 0.0, 1.0); }
    `;
  const layout = t.device.createPipelineLayout({
    bindGroupLayouts: [],
    immediateSize: kImmediateSize
  });
  const pipeline = t.device.createRenderPipeline({
    layout,
    vertex: {
      module: t.device.createShaderModule({ code })
    },
    fragment: {
      module: t.device.createShaderModule({ code }),
      targets: [{ format: 'rgba8unorm' }]
    }
  });

  // Create a bundle that sets immediates (to test leak)
  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats: ['rgba8unorm']
  });
  bundleEncoder.setPipeline(pipeline);
  const immediateData = new Uint8Array(16);
  bundleEncoder.setImmediates(0, immediateData, 0, 16);
  bundleEncoder.draw(3);
  const bundle = bundleEncoder.finish();

  const { encoder, validateFinishAndSubmit } = t.createEncoder('render pass');
  const pass = encoder;

  // Initial setup
  pass.setPipeline(pipeline);
  pass.setImmediates(0, immediateData, 0, 16);

  // Execute bundle - this should invalidate state
  pass.executeBundles([bundle]);

  // Try to draw
  pass.setPipeline(pipeline);
  if (resetImmediates) {
    pass.setImmediates(0, immediateData, 0, 16);
  }
  pass.draw(3);

  validateFinishAndSubmit(resetImmediates, true);
});