/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
This test dedicatedly tests validation of pipeline overridable constants of createRenderPipeline.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kValue } from '../../../util/constants.js';

import { CreateRenderPipelineValidationTest } from './common.js';

export const g = makeTestGroup(CreateRenderPipelineValidationTest);

g.test('identifier,vertex')
  .desc(
    `
Tests calling createRenderPipeline(Async) validation for overridable constants identifiers in vertex state.
`
  )
  .params(u =>
    u //
      .combine('isAsync', [true, false])
      .combineWithParams([
        { vertexConstants: {}, _success: true },
        { vertexConstants: { x: 1, y: 1 }, _success: true },
        { vertexConstants: { x: 1, y: 1, 1: 1, 1000: 1 }, _success: true },
        { vertexConstants: { 'x\0': 1, y: 1 }, _success: false },
        { vertexConstants: { xxx: 1 }, _success: false },
        { vertexConstants: { 1: 1 }, _success: true },
        { vertexConstants: { 2: 1 }, _success: false },
        { vertexConstants: { z: 1 }, _success: false }, // pipeline constant id is specified for z
        { vertexConstants: { w: 1 }, _success: false }, // pipeline constant id is specified for w
        { vertexConstants: { 1: 1, z: 1 }, _success: false }, // pipeline constant id is specified for z
        { vertexConstants: { 数: 1 }, _success: true }, // test non-ASCII
        { vertexConstants: { séquençage: 0 }, _success: false }, // test unicode normalization
      ])
  )
  .fn(t => {
    const { isAsync, vertexConstants, _success } = t.params;

    t.doCreateRenderPipelineTest(isAsync, _success, {
      layout: 'auto',
      vertex: {
        module: t.device.createShaderModule({
          code: `
            override x: f32 = 0.0;
            override y: f32 = 0.0;
            override 数: f32 = 0.0;
            override séquençage: f32 = 0.0;
            @id(1) override z: f32 = 0.0;
            @id(1000) override w: f32 = 1.0;
            @vertex fn main() -> @builtin(position) vec4<f32> {
              return vec4<f32>(x, y, z, w + 数 + séquençage);
            }`,
        }),
        entryPoint: 'main',
        constants: vertexConstants,
      },
      fragment: {
        module: t.device.createShaderModule({
          code: `@fragment fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`,
        }),
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm' }],
      },
    });
  });

g.test('identifier,fragment')
  .desc(
    `
Tests calling createRenderPipeline(Async) validation for overridable constants identifiers in fragment state.
`
  )
  .params(u =>
    u //
      .combine('isAsync', [true, false])
      .combineWithParams([
        { fragmentConstants: {}, _success: true },
        { fragmentConstants: { r: 1, g: 1 }, _success: true },
        { fragmentConstants: { r: 1, g: 1, 1: 1, 1000: 1 }, _success: true },
        { fragmentConstants: { 'r\0': 1 }, _success: false },
        { fragmentConstants: { xxx: 1 }, _success: false },
        { fragmentConstants: { 1: 1 }, _success: true },
        { fragmentConstants: { 2: 1 }, _success: false },
        { fragmentConstants: { b: 1 }, _success: false }, // pipeline constant id is specified for b
        { fragmentConstants: { a: 1 }, _success: false }, // pipeline constant id is specified for a
        { fragmentConstants: { 1: 1, b: 1 }, _success: false }, // pipeline constant id is specified for b
        { fragmentConstants: { 数: 1 }, _success: true }, // test non-ASCII
        { fragmentConstants: { séquençage: 0 }, _success: false }, // test unicode is not normalized
      ])
  )
  .fn(t => {
    const { isAsync, fragmentConstants, _success } = t.params;

    const descriptor = t.getDescriptor({
      fragmentShaderCode: `
        override r: f32 = 0.0;
        override g: f32 = 0.0;
        override 数: f32 = 0.0;
        override sequencage: f32 = 0.0;
        @id(1) override b: f32 = 0.0;
        @id(1000) override a: f32 = 0.0;
        @fragment fn main()
            -> @location(0) vec4<f32> {
            return vec4<f32>(r, g, b, a + 数 + sequencage);
        }`,
      fragmentConstants,
    });

    t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
  });

g.test('uninitialized,vertex')
  .desc(
    `
Tests calling createRenderPipeline(Async) validation for uninitialized overridable constants in vertex state.
`
  )
  .params(u =>
    u //
      .combine('isAsync', [true, false])
      .combineWithParams([
        { vertexConstants: {}, _success: false },
        { vertexConstants: { x: 1, y: 1 }, _success: false }, // z is missing
        { vertexConstants: { x: 1, z: 1 }, _success: true },
        { vertexConstants: { x: 1, y: 1, z: 1, w: 1 }, _success: true },
      ])
  )
  .fn(t => {
    const { isAsync, vertexConstants, _success } = t.params;

    t.doCreateRenderPipelineTest(isAsync, _success, {
      layout: 'auto',
      vertex: {
        module: t.device.createShaderModule({
          code: `
            override x: f32;
            override y: f32 = 0.0;
            override z: f32;
            override w: f32 = 1.0;
            @vertex fn main() -> @builtin(position) vec4<f32> {
              return vec4<f32>(x, y, z, w);
            }`,
        }),
        entryPoint: 'main',
        constants: vertexConstants,
      },
      fragment: {
        module: t.device.createShaderModule({
          code: `@fragment fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`,
        }),
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm' }],
      },
    });
  });

g.test('uninitialized,fragment')
  .desc(
    `
Tests calling createRenderPipeline(Async) validation for uninitialized overridable constants in fragment state.
`
  )
  .params(u =>
    u //
      .combine('isAsync', [true, false])
      .combineWithParams([
        { fragmentConstants: {}, _success: false },
        { fragmentConstants: { r: 1, g: 1 }, _success: false }, // b is missing
        { fragmentConstants: { r: 1, b: 1 }, _success: true },
        { fragmentConstants: { r: 1, g: 1, b: 1, a: 1 }, _success: true },
      ])
  )
  .fn(t => {
    const { isAsync, fragmentConstants, _success } = t.params;

    const descriptor = t.getDescriptor({
      fragmentShaderCode: `
        override r: f32;
        override g: f32 = 0.0;
        override b: f32;
        override a: f32 = 0.0;
        @fragment fn main()
            -> @location(0) vec4<f32> {
            return vec4<f32>(r, g, b, a);
        }
          `,
      fragmentConstants,
    });

    t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
  });

g.test('value,type_error,vertex')
  .desc(
    `
Tests calling createRenderPipeline(Async) validation for invalid constant values like inf, NaN will results in TypeError.
`
  )
  .params(u =>
    u //
      .combine('isAsync', [true, false])
      .combineWithParams([
        { vertexConstants: { cf: 1 }, _success: true }, // control
        { vertexConstants: { cf: NaN }, _success: false },
        { vertexConstants: { cf: Number.POSITIVE_INFINITY }, _success: false },
        { vertexConstants: { cf: Number.NEGATIVE_INFINITY }, _success: false },
      ])
  )
  .fn(t => {
    const { isAsync, vertexConstants, _success } = t.params;

    t.doCreateRenderPipelineTest(
      isAsync,
      _success,
      {
        layout: 'auto',
        vertex: {
          module: t.device.createShaderModule({
            code: `
            override cf: f32 = 0.0;
            @vertex fn main() -> @builtin(position) vec4<f32> {
              _ = cf;
              return vec4<f32>(0.0, 0.0, 0.0, 1.0);
            }`,
          }),
          entryPoint: 'main',
          constants: vertexConstants,
        },
        fragment: {
          module: t.device.createShaderModule({
            code: `@fragment fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`,
          }),
          entryPoint: 'main',
          targets: [{ format: 'rgba8unorm' }],
        },
      },
      'TypeError'
    );
  });

g.test('value,type_error,fragment')
  .desc(
    `
Tests calling createRenderPipeline(Async) validation for invalid constant values like inf, NaN will results in TypeError.
`
  )
  .params(u =>
    u //
      .combine('isAsync', [true, false])
      .combineWithParams([
        { fragmentConstants: { cf: 1 }, _success: true }, // control
        { fragmentConstants: { cf: NaN }, _success: false },
        { fragmentConstants: { cf: Number.POSITIVE_INFINITY }, _success: false },
        { fragmentConstants: { cf: Number.NEGATIVE_INFINITY }, _success: false },
      ])
  )
  .fn(t => {
    const { isAsync, fragmentConstants, _success } = t.params;

    const descriptor = t.getDescriptor({
      fragmentShaderCode: `
        override cf: f32 = 0.0;
        @fragment fn main()
            -> @location(0) vec4<f32> {
            _ = cf;
            return vec4<f32>(1.0, 1.0, 1.0, 1.0);
        }
          `,
      fragmentConstants,
    });

    t.doCreateRenderPipelineTest(isAsync, _success, descriptor, 'TypeError');
  });

g.test('value,validation_error,vertex')
  .desc(
    `
Tests calling createRenderPipeline(Async) validation for unrepresentable constant values in vertex stage.

TODO(#2060): test with last_f64_castable.
`
  )
  .params(u =>
    u //
      .combine('isAsync', [true, false])
      .combineWithParams([
        { vertexConstants: { cu: kValue.u32.min }, _success: true },
        { vertexConstants: { cu: kValue.u32.min - 1 }, _success: false },
        { vertexConstants: { cu: kValue.u32.max }, _success: true },
        { vertexConstants: { cu: kValue.u32.max + 1 }, _success: false },
        { vertexConstants: { ci: kValue.i32.negative.min }, _success: true },
        { vertexConstants: { ci: kValue.i32.negative.min - 1 }, _success: false },
        { vertexConstants: { ci: kValue.i32.positive.max }, _success: true },
        { vertexConstants: { ci: kValue.i32.positive.max + 1 }, _success: false },
        { vertexConstants: { cf: kValue.f32.negative.min }, _success: true },
        {
          vertexConstants: { cf: kValue.f32.negative.first_non_castable_pipeline_override },
          _success: false,
        },
        { vertexConstants: { cf: kValue.f32.positive.max }, _success: true },
        {
          vertexConstants: { cf: kValue.f32.positive.first_non_castable_pipeline_override },
          _success: false,
        },
        // Conversion to boolean can't fail
        { vertexConstants: { cb: Number.MAX_VALUE }, _success: true },
        { vertexConstants: { cb: kValue.i32.negative.min - 1 }, _success: true },
      ])
  )
  .fn(t => {
    const { isAsync, vertexConstants, _success } = t.params;

    t.doCreateRenderPipelineTest(isAsync, _success, {
      layout: 'auto',
      vertex: {
        module: t.device.createShaderModule({
          code: `
          override cb: bool = false;
          override cu: u32 = 0u;
          override ci: i32 = 0;
          override cf: f32 = 0.0;
          @vertex fn main() -> @builtin(position) vec4<f32> {
            _ = cb;
            _ = cu;
            _ = ci;
            _ = cf;
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
          }`,
        }),
        entryPoint: 'main',
        constants: vertexConstants,
      },
      fragment: {
        module: t.device.createShaderModule({
          code: `@fragment fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`,
        }),
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm' }],
      },
    });
  });

g.test('value,validation_error,fragment')
  .desc(
    `
Tests calling createRenderPipeline(Async) validation for unrepresentable constant values in fragment stage.

TODO(#2060): test with last_f64_castable.
`
  )
  .params(u =>
    u //
      .combine('isAsync', [true, false])
      .combineWithParams([
        { fragmentConstants: { cu: kValue.u32.min }, _success: true },
        { fragmentConstants: { cu: kValue.u32.min - 1 }, _success: false },
        { fragmentConstants: { cu: kValue.u32.max }, _success: true },
        { fragmentConstants: { cu: kValue.u32.max + 1 }, _success: false },
        { fragmentConstants: { ci: kValue.i32.negative.min }, _success: true },
        { fragmentConstants: { ci: kValue.i32.negative.min - 1 }, _success: false },
        { fragmentConstants: { ci: kValue.i32.positive.max }, _success: true },
        { fragmentConstants: { ci: kValue.i32.positive.max + 1 }, _success: false },
        { fragmentConstants: { cf: kValue.f32.negative.min }, _success: true },
        {
          fragmentConstants: { cf: kValue.f32.negative.first_non_castable_pipeline_override },
          _success: false,
        },
        { fragmentConstants: { cf: kValue.f32.positive.max }, _success: true },
        {
          fragmentConstants: { cf: kValue.f32.positive.first_non_castable_pipeline_override },
          _success: false,
        },
        // Conversion to boolean can't fail
        { fragmentConstants: { cb: Number.MAX_VALUE }, _success: true },
        { fragmentConstants: { cb: kValue.i32.negative.min - 1 }, _success: true },
      ])
  )
  .fn(t => {
    const { isAsync, fragmentConstants, _success } = t.params;

    const descriptor = t.getDescriptor({
      fragmentShaderCode: `
        override cb: bool = false;
        override cu: u32 = 0u;
        override ci: i32 = 0;
        override cf: f32 = 0.0;
        @fragment fn main()
            -> @location(0) vec4<f32> {
            _ = cb;
            _ = cu;
            _ = ci;
            _ = cf;
            return vec4<f32>(1.0, 1.0, 1.0, 1.0);
        }
          `,
      fragmentConstants,
    });

    t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
  });

g.test('value,validation_error,f16,vertex')
  .desc(
    `
Tests calling createRenderPipeline(Async) validation for unrepresentable f16 constant values in vertex stage.

TODO(#2060): Tighten the cases around the valid/invalid boundary once we have WGSL spec
clarity on whether values like f16.positive.last_f64_castable would be valid. See issue.
`
  )
  .params(u =>
    u //
      .combine('isAsync', [true, false])
      .combineWithParams([
        { vertexConstants: { cf16: kValue.f16.negative.min }, _success: true },
        {
          vertexConstants: { cf16: kValue.f16.negative.first_non_castable_pipeline_override },
          _success: false,
        },
        { vertexConstants: { cf16: kValue.f16.positive.max }, _success: true },
        {
          vertexConstants: { cf16: kValue.f16.positive.first_non_castable_pipeline_override },
          _success: false,
        },
        { vertexConstants: { cf16: kValue.f32.negative.min }, _success: false },
        { vertexConstants: { cf16: kValue.f32.positive.max }, _success: false },
        {
          vertexConstants: { cf16: kValue.f32.negative.first_non_castable_pipeline_override },
          _success: false,
        },
        {
          vertexConstants: { cf16: kValue.f32.positive.first_non_castable_pipeline_override },
          _success: false,
        },
      ])
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .fn(t => {
    const { isAsync, vertexConstants, _success } = t.params;

    t.doCreateRenderPipelineTest(isAsync, _success, {
      layout: 'auto',
      vertex: {
        module: t.device.createShaderModule({
          code: `
          enable f16;

          override cf16: f16 = 0.0h;
          @vertex fn main() -> @builtin(position) vec4<f32> {
            _ = cf16;
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
          }`,
        }),
        entryPoint: 'main',
        constants: vertexConstants,
      },
      fragment: {
        module: t.device.createShaderModule({
          code: `@fragment fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`,
        }),
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm' }],
      },
    });
  });

g.test('value,validation_error,f16,fragment')
  .desc(
    `
Tests calling createRenderPipeline(Async) validation for unrepresentable f16 constant values in fragment stage.

TODO(#2060): Tighten the cases around the valid/invalid boundary once we have WGSL spec
clarity on whether values like f16.positive.last_f64_castable would be valid. See issue.
`
  )
  .beforeAllSubcases(t => {
    t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
  })
  .params(u =>
    u //
      .combine('isAsync', [true, false])
      .combineWithParams([
        { fragmentConstants: { cf16: kValue.f16.negative.min }, _success: true },
        {
          fragmentConstants: { cf16: kValue.f16.negative.first_non_castable_pipeline_override },
          _success: false,
        },
        { fragmentConstants: { cf16: kValue.f16.positive.max }, _success: true },
        {
          fragmentConstants: { cf16: kValue.f16.positive.first_non_castable_pipeline_override },
          _success: false,
        },
        { fragmentConstants: { cf16: kValue.f32.negative.min }, _success: false },
        { fragmentConstants: { cf16: kValue.f32.positive.max }, _success: false },
        {
          fragmentConstants: { cf16: kValue.f32.negative.first_non_castable_pipeline_override },
          _success: false,
        },
        {
          fragmentConstants: { cf16: kValue.f32.positive.first_non_castable_pipeline_override },
          _success: false,
        },
      ])
  )
  .fn(t => {
    const { isAsync, fragmentConstants, _success } = t.params;

    const descriptor = t.getDescriptor({
      fragmentShaderCode: `
        enable f16;

        override cf16: f16 = 0.0h;
        @fragment fn main()
            -> @location(0) vec4<f32> {
            _ = cf16;
            return vec4<f32>(1.0, 1.0, 1.0, 1.0);
        }
          `,
      fragmentConstants,
    });

    t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
  });
