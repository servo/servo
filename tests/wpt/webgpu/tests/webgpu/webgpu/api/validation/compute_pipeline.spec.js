/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
createComputePipeline and createComputePipelineAsync validation tests.

Note: entry point matching tests are in shader_module/entry_point.spec.ts
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { keysOf } from '../../../common/util/data_tables.js';
import { kValue } from '../../util/constants.js';
import { getShaderWithEntryPoint } from '../../util/shader.js';

import {
  kAPIResources,
  getWGSLShaderForResource,
  getAPIBindGroupLayoutForResource,
  doResourcesMatch } from
'./utils.js';
import { ValidationTest } from './validation_test.js';

class F extends ValidationTest {
  getShaderModule(
  shaderStage = 'compute',
  entryPoint = 'main')
  {
    return this.device.createShaderModule({
      code: getShaderWithEntryPoint(shaderStage, entryPoint)
    });
  }
}

export const g = makeTestGroup(F);

g.test('basic').
desc(
  `
Control case for createComputePipeline and createComputePipelineAsync.
Call the API with valid compute shader and matching valid entryPoint, making sure that the test function working well.
`
).
params((u) => u.combine('isAsync', [true, false])).
fn((t) => {
  const { isAsync } = t.params;
  t.doCreateComputePipelineTest(isAsync, true, {
    layout: 'auto',
    compute: { module: t.getShaderModule('compute', 'main'), entryPoint: 'main' }
  });
});

g.test('shader_module,invalid').
desc(
  `
Tests calling createComputePipeline(Async) with a invalid compute shader, and check that the APIs catch this error.
`
).
params((u) => u.combine('isAsync', [true, false])).
fn((t) => {
  const { isAsync } = t.params;
  t.doCreateComputePipelineTest(isAsync, false, {
    layout: 'auto',
    compute: {
      module: t.createInvalidShaderModule(),
      entryPoint: 'main'
    }
  });
});

g.test('shader_module,compute').
desc(
  `
Tests calling createComputePipeline(Async) with valid but different stage shader and matching entryPoint,
and check that the APIs only accept compute shader.
`
).
params((u) =>
u //
.combine('isAsync', [true, false]).
combine('shaderModuleStage', ['compute', 'vertex', 'fragment'])
).
fn((t) => {
  const { isAsync, shaderModuleStage } = t.params;
  const descriptor = {
    layout: 'auto',
    compute: {
      module: t.getShaderModule(shaderModuleStage, 'main'),
      entryPoint: 'main'
    }
  };
  t.doCreateComputePipelineTest(isAsync, shaderModuleStage === 'compute', descriptor);
});

g.test('shader_module,device_mismatch').
desc(
  'Tests createComputePipeline(Async) cannot be called with a shader module created from another device'
).
paramsSubcasesOnly((u) => u.combine('isAsync', [true, false]).combine('mismatched', [true, false])).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { isAsync, mismatched } = t.params;

  const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

  const module = sourceDevice.createShaderModule({
    code: '@compute @workgroup_size(1) fn main() {}'
  });

  const descriptor = {
    layout: 'auto',
    compute: {
      module,
      entryPoint: 'main'
    }
  };

  t.doCreateComputePipelineTest(isAsync, !mismatched, descriptor);
});

g.test('pipeline_layout,device_mismatch').
desc(
  'Tests createComputePipeline(Async) cannot be called with a pipeline layout created from another device'
).
paramsSubcasesOnly((u) => u.combine('isAsync', [true, false]).combine('mismatched', [true, false])).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { isAsync, mismatched } = t.params;
  const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

  const layout = sourceDevice.createPipelineLayout({ bindGroupLayouts: [] });

  const descriptor = {
    layout,
    compute: {
      module: t.getShaderModule('compute', 'main'),
      entryPoint: 'main'
    }
  };

  t.doCreateComputePipelineTest(isAsync, !mismatched, descriptor);
});

g.test('limits,workgroup_storage_size').
desc(
  `
Tests calling createComputePipeline(Async) validation for compute using <= device.limits.maxComputeWorkgroupStorageSize bytes of workgroup storage.
`
).
params((u) =>
u //
.combine('isAsync', [true, false]).
combineWithParams([
{ type: 'vec4<f32>', _typeSize: 16 },
{ type: 'mat4x4<f32>', _typeSize: 64 }]
).
beginSubcases().
combine('countDeltaFromLimit', [0, 1])
).
fn((t) => {
  const { isAsync, type, _typeSize, countDeltaFromLimit } = t.params;
  const countAtLimit = Math.floor(t.device.limits.maxComputeWorkgroupStorageSize / _typeSize);
  const count = countAtLimit + countDeltaFromLimit;

  const descriptor = {
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: `
          var<workgroup> data: array<${type}, ${count}>;
          @compute @workgroup_size(64) fn main () {
            _ = data;
          }
          `
      }),
      entryPoint: 'main'
    }
  };
  t.doCreateComputePipelineTest(isAsync, count <= countAtLimit, descriptor);
});

g.test('limits,invocations_per_workgroup').
desc(
  `
Tests calling createComputePipeline(Async) validation for compute using <= device.limits.maxComputeInvocationsPerWorkgroup per workgroup.
`
).
params((u) =>
u //
.combine('isAsync', [true, false]).
combine('size', [
// Assume maxComputeWorkgroupSizeX/Y >= 129, maxComputeWorkgroupSizeZ >= 33
[128, 1, 2],
[129, 1, 2],
[2, 128, 1],
[2, 129, 1],
[1, 8, 32],
[1, 8, 33]]
)
).
fn((t) => {
  const { isAsync, size } = t.params;

  const descriptor = {
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: `
          @compute @workgroup_size(${size.join(',')}) fn main () {
          }
          `
      }),
      entryPoint: 'main'
    }
  };

  t.doCreateComputePipelineTest(
    isAsync,
    size[0] * size[1] * size[2] <= t.device.limits.maxComputeInvocationsPerWorkgroup,
    descriptor
  );
});

g.test('limits,invocations_per_workgroup,each_component').
desc(
  `
Tests calling createComputePipeline(Async) validation for compute workgroup_size attribute has each component no more than their limits.
`
).
params((u) =>
u //
.combine('isAsync', [true, false]).
combine('size', [
// Assume maxComputeInvocationsPerWorkgroup >= 256
[64],
[256, 1, 1],
[257, 1, 1],
[1, 256, 1],
[1, 257, 1],
[1, 1, 63],
[1, 1, 64],
[1, 1, 65]]
)
).
fn((t) => {
  const { isAsync, size } = t.params;

  const descriptor = {
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: `
          @compute @workgroup_size(${size.join(',')}) fn main () {
          }
          `
      }),
      entryPoint: 'main'
    }
  };

  const workgroupX = size[0];
  const workgroupY = size[1] ?? 1;
  const workgroupZ = size[2] ?? 1;

  const _success =
  workgroupX <= t.device.limits.maxComputeWorkgroupSizeX &&
  workgroupY <= t.device.limits.maxComputeWorkgroupSizeY &&
  workgroupZ <= t.device.limits.maxComputeWorkgroupSizeZ;
  t.doCreateComputePipelineTest(isAsync, _success, descriptor);
});

g.test('overrides,identifier').
desc(
  `
Tests calling createComputePipeline(Async) validation for overridable constants identifiers.
`
).
params((u) =>
u //
.combine('isAsync', [true, false]).
combineWithParams([
{ constants: {}, _success: true },
{ constants: { c0: 0 }, _success: true },
{ constants: { c0: 0, c1: 1 }, _success: true },
{ constants: { 'c0\0': 0 }, _success: false },
{ constants: { c9: 0 }, _success: false },
{ constants: { 1: 0 }, _success: true },
{ constants: { c3: 0 }, _success: false }, // pipeline constant id is specified for c3
{ constants: { 2: 0 }, _success: false },
{ constants: { 1000: 0 }, _success: true },
{ constants: { 9999: 0 }, _success: false },
{ constants: { 1000: 0, c2: 0 }, _success: false },
{ constants: { 数: 0 }, _success: true },
{ constants: { séquençage: 0 }, _success: false } // test unicode is not normalized
])
).
fn((t) => {
  const { isAsync, constants, _success } = t.params;

  const descriptor = {
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: `
            override c0: bool = true;      // type: bool
            override c1: u32 = 0u;          // default override
            override 数: u32 = 0u;          // non-ASCII
            override séquençage: u32 = 0u;  // normalizable unicode (WGSL does not normalize)
            @id(1000) override c2: u32 = 10u;  // default
            @id(1) override c3: u32 = 11u;     // default
            @compute @workgroup_size(1) fn main () {
              // make sure the overridable constants are not optimized out
              _ = u32(c0);
              _ = u32(c1);
              _ = u32(c2 + séquençage);
              _ = u32(c3 + 数);
            }`
      }),
      entryPoint: 'main',
      constants
    }
  };

  t.doCreateComputePipelineTest(isAsync, _success, descriptor);
});

g.test('overrides,uninitialized').
desc(
  `
Tests calling createComputePipeline(Async) validation for uninitialized overridable constants.
`
).
params((u) =>
u //
.combine('isAsync', [true, false]).
combineWithParams([
{ constants: {}, _success: false },
{ constants: { c0: 0, c2: 0, c8: 0 }, _success: false }, // c5 is missing
{ constants: { c0: 0, c2: 0, c5: 0, c8: 0 }, _success: true },
{ constants: { c0: 0, c2: 0, c5: 0, c8: 0, c1: 0 }, _success: true }]
)
).
fn((t) => {
  const { isAsync, constants, _success } = t.params;

  const descriptor = {
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: `
            override c0: bool;              // type: bool
            override c1: bool = false;      // default override
            override c2: f32;               // type: float32
            override c3: f32 = 0.0;         // default override
            override c4: f32 = 4.0;         // default
            override c5: i32;               // type: int32
            override c6: i32 = 0;           // default override
            override c7: i32 = 7;           // default
            override c8: u32;               // type: uint32
            override c9: u32 = 0u;          // default override
            @id(1000) override c10: u32 = 10u;  // default
            @compute @workgroup_size(1) fn main () {
              // make sure the overridable constants are not optimized out
              _ = u32(c0);
              _ = u32(c1);
              _ = u32(c2);
              _ = u32(c3);
              _ = u32(c4);
              _ = u32(c5);
              _ = u32(c6);
              _ = u32(c7);
              _ = u32(c8);
              _ = u32(c9);
              _ = u32(c10);
            }`
      }),
      entryPoint: 'main',
      constants
    }
  };

  t.doCreateComputePipelineTest(isAsync, _success, descriptor);
});

g.test('overrides,value,type_error').
desc(
  `
Tests calling createComputePipeline(Async) validation for constant values like inf, NaN will results in TypeError.
`
).
params((u) =>
u //
.combine('isAsync', [true, false]).
combineWithParams([
{ constants: { cf: 1 }, _success: true }, // control
{ constants: { cf: NaN }, _success: false },
{ constants: { cf: Number.POSITIVE_INFINITY }, _success: false },
{ constants: { cf: Number.NEGATIVE_INFINITY }, _success: false }]
)
).
fn((t) => {
  const { isAsync, constants, _success } = t.params;

  const descriptor = {
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: `
            override cf: f32 = 0.0;
            @compute @workgroup_size(1) fn main () {
              _ = cf;
            }`
      }),
      entryPoint: 'main',
      constants
    }
  };

  t.doCreateComputePipelineTest(isAsync, _success, descriptor, 'TypeError');
});

g.test('overrides,value,validation_error').
desc(
  `
Tests calling createComputePipeline(Async) validation for unrepresentable constant values in compute stage.

TODO(#2060): test with last_castable_pipeline_override.
`
).
params((u) =>
u //
.combine('isAsync', [true, false]).
combineWithParams([
{ constants: { cu: kValue.u32.min }, _success: true },
{ constants: { cu: kValue.u32.min - 1 }, _success: false },
{ constants: { cu: kValue.u32.max }, _success: true },
{ constants: { cu: kValue.u32.max + 1 }, _success: false },
{ constants: { ci: kValue.i32.negative.min }, _success: true },
{ constants: { ci: kValue.i32.negative.min - 1 }, _success: false },
{ constants: { ci: kValue.i32.positive.max }, _success: true },
{ constants: { ci: kValue.i32.positive.max + 1 }, _success: false },
{ constants: { cf: kValue.f32.negative.min }, _success: true },
{
  constants: { cf: kValue.f32.negative.first_non_castable_pipeline_override },
  _success: false
},
{ constants: { cf: kValue.f32.positive.max }, _success: true },
{
  constants: { cf: kValue.f32.positive.first_non_castable_pipeline_override },
  _success: false
},
// Conversion to boolean can't fail
{ constants: { cb: Number.MAX_VALUE }, _success: true },
{ constants: { cb: kValue.i32.negative.min - 1 }, _success: true }]
)
).
fn((t) => {
  const { isAsync, constants, _success } = t.params;
  const descriptor = {
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: `
          override cb: bool = false;
          override cu: u32 = 0u;
          override ci: i32 = 0;
          override cf: f32 = 0.0;
          @compute @workgroup_size(1) fn main () {
            _ = cb;
            _ = cu;
            _ = ci;
            _ = cf;
          }`
      }),
      entryPoint: 'main',
      constants
    }
  };

  t.doCreateComputePipelineTest(isAsync, _success, descriptor);
});

g.test('overrides,value,validation_error,f16').
desc(
  `
Tests calling createComputePipeline(Async) validation for unrepresentable f16 constant values in compute stage.

TODO(#2060): Tighten the cases around the valid/invalid boundary once we have WGSL spec
clarity on whether values like f16.positive.last_castable_pipeline_override would be valid. See issue.
`
).
params((u) =>
u //
.combine('isAsync', [true, false]).
combineWithParams([
{ constants: { cf16: kValue.f16.negative.min }, _success: true },
{
  constants: { cf16: kValue.f16.negative.first_non_castable_pipeline_override },
  _success: false
},
{ constants: { cf16: kValue.f16.positive.max }, _success: true },
{
  constants: { cf16: kValue.f16.positive.first_non_castable_pipeline_override },
  _success: false
},
{ constants: { cf16: kValue.f32.negative.min }, _success: false },
{ constants: { cf16: kValue.f32.positive.max }, _success: false },
{
  constants: { cf16: kValue.f32.negative.first_non_castable_pipeline_override },
  _success: false
},
{
  constants: { cf16: kValue.f32.positive.first_non_castable_pipeline_override },
  _success: false
}]
)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn((t) => {
  const { isAsync, constants, _success } = t.params;

  const descriptor = {
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: `
          enable f16;

          override cf16: f16 = 0.0h;
          @compute @workgroup_size(1) fn main () {
            _ = cf16;
          }`
      }),
      entryPoint: 'main',
      constants
    }
  };

  t.doCreateComputePipelineTest(isAsync, _success, descriptor);
});

const kOverridesWorkgroupSizeShaders = {
  u32: `
override x: u32 = 1u;
override y: u32 = 1u;
override z: u32 = 1u;
@compute @workgroup_size(x, y, z) fn main () {
  _ = 0u;
}
`,
  i32: `
override x: i32 = 1;
override y: i32 = 1;
override z: i32 = 1;
@compute @workgroup_size(x, y, z) fn main () {
  _ = 0u;
}
`
};

g.test('overrides,workgroup_size').
desc(
  `
Tests calling createComputePipeline(Async) validation for overridable constants used for workgroup size.
`
).
params((u) =>
u //
.combine('isAsync', [true, false]).
combine('type', ['u32', 'i32']).
combineWithParams([
{ constants: {}, _success: true },
{ constants: { x: 0, y: 0, z: 0 }, _success: false },
{ constants: { x: 1, y: -1, z: 1 }, _success: false },
{ constants: { x: 1, y: 0, z: 0 }, _success: false },
{ constants: { x: 16, y: 1, z: 1 }, _success: true }]
)
).
fn((t) => {
  const { isAsync, type, constants, _success } = t.params;

  const descriptor = {
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code: kOverridesWorkgroupSizeShaders[type]
      }),
      entryPoint: 'main',
      constants
    }
  };

  t.doCreateComputePipelineTest(isAsync, _success, descriptor);
});

g.test('overrides,workgroup_size,limits').
desc(
  `
Tests calling createComputePipeline(Async) validation for overridable constants for workgroupSize exceeds device limits.
`
).
params((u) =>
u //
.combine('isAsync', [true, false]).
combine('type', ['u32', 'i32'])
).
fn((t) => {
  const { isAsync, type } = t.params;

  const limits = t.device.limits;

  const testFn = (x, y, z, _success) => {
    const descriptor = {
      layout: 'auto',
      compute: {
        module: t.device.createShaderModule({
          code: kOverridesWorkgroupSizeShaders[type]
        }),
        entryPoint: 'main',
        constants: {
          x,
          y,
          z
        }
      }
    };

    t.doCreateComputePipelineTest(isAsync, _success, descriptor);
  };

  testFn(limits.maxComputeWorkgroupSizeX, 1, 1, true);
  testFn(limits.maxComputeWorkgroupSizeX + 1, 1, 1, false);
  testFn(1, limits.maxComputeWorkgroupSizeY, 1, true);
  testFn(1, limits.maxComputeWorkgroupSizeY + 1, 1, false);
  testFn(1, 1, limits.maxComputeWorkgroupSizeZ, true);
  testFn(1, 1, limits.maxComputeWorkgroupSizeZ + 1, false);
  testFn(
    limits.maxComputeWorkgroupSizeX,
    limits.maxComputeWorkgroupSizeY,
    limits.maxComputeWorkgroupSizeZ,
    limits.maxComputeWorkgroupSizeX *
    limits.maxComputeWorkgroupSizeY *
    limits.maxComputeWorkgroupSizeZ <=
    limits.maxComputeInvocationsPerWorkgroup
  );
});

g.test('overrides,workgroup_size,limits,workgroup_storage_size').
desc(
  `
Tests calling createComputePipeline(Async) validation for overridable constants for workgroupStorageSize exceeds device limits.
`
).
params((u) =>
u //
.combine('isAsync', [true, false])
).
fn((t) => {
  const { isAsync } = t.params;

  const limits = t.device.limits;

  const kVec4Size = 16;
  const maxVec4Count = limits.maxComputeWorkgroupStorageSize / kVec4Size;
  const kMat4Size = 64;
  const maxMat4Count = limits.maxComputeWorkgroupStorageSize / kMat4Size;

  const testFn = (vec4Count, mat4Count, _success) => {
    const descriptor = {
      layout: 'auto',
      compute: {
        module: t.device.createShaderModule({
          code: `
              override a: u32;
              override b: u32;
              ${vec4Count <= 0 ? '' : 'var<workgroup> vec4_data: array<vec4<f32>, a>;'}
              ${mat4Count <= 0 ? '' : 'var<workgroup> mat4_data: array<mat4x4<f32>, b>;'}
              @compute @workgroup_size(1) fn main() {
                ${vec4Count <= 0 ? '' : '_ = vec4_data[0];'}
                ${mat4Count <= 0 ? '' : '_ = mat4_data[0];'}
              }`
        }),
        entryPoint: 'main',
        constants: {
          a: vec4Count,
          b: mat4Count
        }
      }
    };

    t.doCreateComputePipelineTest(isAsync, _success, descriptor);
  };

  testFn(1, 1, true);
  testFn(maxVec4Count + 1, 0, false);
  testFn(0, maxMat4Count + 1, false);
});

g.test('resource_compatibility').
desc(
  'Tests validation of resource (bind group) compatibility between pipeline layout and WGSL shader'
).
params((u) =>
u //
.combine('apiResource', keysOf(kAPIResources)).
beginSubcases().
combine('isAsync', [true, false]).
combine('wgslResource', keysOf(kAPIResources))
).
fn((t) => {
  const apiResource = kAPIResources[t.params.apiResource];
  const wgslResource = kAPIResources[t.params.wgslResource];
  t.skipIf(
    wgslResource.storageTexture !== undefined &&
    wgslResource.storageTexture.access !== 'write-only' &&
    !t.hasLanguageFeature('readonly_and_readwrite_storage_textures'),
    'Storage textures require language feature'
  );

  const layout = t.device.createPipelineLayout({
    bindGroupLayouts: [
    getAPIBindGroupLayoutForResource(t.device, GPUShaderStage.COMPUTE, apiResource)]

  });

  const descriptor = {
    layout,
    compute: {
      module: t.device.createShaderModule({
        code: getWGSLShaderForResource('compute', wgslResource)
      }),
      entryPoint: 'main'
    }
  };
  t.doCreateComputePipelineTest(
    t.params.isAsync,
    doResourcesMatch(apiResource, wgslResource),
    descriptor
  );
});