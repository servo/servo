/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
setBindGroup validation tests.

TODO: merge these notes and implement.
> (Note: If there are errors with using certain binding types in certain passes, test those in the file for that pass type, not here.)
>
> - state tracking (probably separate file)
>     - x= {compute pass, render pass}
>     - {null, compatible, incompatible} current pipeline (should have no effect without draw/dispatch)
>     - setBindGroup in different orders (e.g. 0,1,2 vs 2,0,1)
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { range, unreachable } from '../../../../../common/util/util.js';
import {
  kBufferBindingTypes,
  kMinDynamicBufferOffsetAlignment,
  kLimitInfo,
} from '../../../../capability_info.js';
import { kResourceStates } from '../../../../gpu_test.js';
import { kProgrammableEncoderTypes } from '../../../../util/command_buffer_maker.js';
import { ValidationTest } from '../../validation_test.js';

class F extends ValidationTest {
  encoderTypeToStageFlag(encoderType) {
    switch (encoderType) {
      case 'compute pass':
        return GPUShaderStage.COMPUTE;
      case 'render pass':
      case 'render bundle':
        return GPUShaderStage.FRAGMENT;
      default:
        unreachable('Unknown encoder type');
    }
  }

  createBindingResourceWithState(resourceType, state) {
    switch (resourceType) {
      case 'texture': {
        const texture = this.createTextureWithState('valid');
        const view = texture.createView();
        if (state === 'destroyed') {
          texture.destroy();
        }
        return view;
      }
      case 'buffer':
        return {
          buffer: this.createBufferWithState(state, {
            size: 4,
            usage: GPUBufferUsage.STORAGE,
          }),
        };
      default:
        unreachable('unknown resource type');
    }
  }

  /**
   * If state is 'invalid', creates an invalid bind group with valid resources.
   * If state is 'destroyed', creates a valid bind group with destroyed resources.
   */
  createBindGroup(state, resourceType, encoderType, indices) {
    if (state === 'invalid') {
      this.device.pushErrorScope('validation');
      indices = new Array(indices.length + 1).fill(0);
    }

    const layout = this.device.createBindGroupLayout({
      entries: indices.map(binding => ({
        binding,
        visibility: this.encoderTypeToStageFlag(encoderType),
        ...(resourceType === 'buffer' ? { buffer: { type: 'storage' } } : { texture: {} }),
      })),
    });
    const bindGroup = this.device.createBindGroup({
      layout,
      entries: indices.map(binding => ({
        binding,
        resource: this.createBindingResourceWithState(
          resourceType,
          state === 'destroyed' ? state : 'valid'
        ),
      })),
    });

    if (state === 'invalid') {
      void this.device.popErrorScope();
    }
    return bindGroup;
  }
}

export const g = makeTestGroup(F);

g.test('state_and_binding_index')
  .desc('Tests that setBindGroup correctly handles {valid, invalid, destroyed} bindGroups.')
  .params(u =>
    u
      .combine('encoderType', kProgrammableEncoderTypes)
      .combine('state', kResourceStates)
      .combine('resourceType', ['buffer', 'texture'])
  )
  .fn(t => {
    const { encoderType, state, resourceType } = t.params;
    const maxBindGroups = t.device.limits.maxBindGroups;

    function runTest(index) {
      const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType);
      encoder.setBindGroup(index, t.createBindGroup(state, resourceType, encoderType, [index]));

      validateFinishAndSubmit(state !== 'invalid' && index < maxBindGroups, state !== 'destroyed');
    }

    // MAINTENANCE_TODO: move to subcases() once we can query the device limits
    for (const index of [1, maxBindGroups - 1, maxBindGroups]) {
      t.debug(`test bind group index ${index}`);
      runTest(index);
    }
  });

g.test('bind_group,device_mismatch')
  .desc(
    `
    Tests setBindGroup cannot be called with a bind group created from another device
    - x= setBindGroup {sequence overload, Uint32Array overload}
    `
  )
  .params(u =>
    u
      .combine('encoderType', kProgrammableEncoderTypes)
      .beginSubcases()
      .combine('useU32Array', [true, false])
      .combine('mismatched', [true, false])
  )
  .beforeAllSubcases(t => {
    t.selectMismatchedDeviceOrSkipTestCase(undefined);
  })
  .fn(t => {
    const { encoderType, useU32Array, mismatched } = t.params;
    const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

    const buffer = sourceDevice.createBuffer({
      size: 4,
      usage: GPUBufferUsage.STORAGE,
    });

    const layout = sourceDevice.createBindGroupLayout({
      entries: [
        {
          binding: 0,
          visibility: t.encoderTypeToStageFlag(encoderType),
          buffer: { type: 'storage', hasDynamicOffset: useU32Array },
        },
      ],
    });

    const bindGroup = sourceDevice.createBindGroup({
      layout,
      entries: [
        {
          binding: 0,
          resource: { buffer },
        },
      ],
    });

    const { encoder, validateFinish } = t.createEncoder(encoderType);
    if (useU32Array) {
      encoder.setBindGroup(0, bindGroup, new Uint32Array([0]), 0, 1);
    } else {
      encoder.setBindGroup(0, bindGroup);
    }
    validateFinish(!mismatched);
  });

g.test('dynamic_offsets_passed_but_not_expected')
  .desc('Tests that setBindGroup correctly errors on unexpected dynamicOffsets.')
  .params(u => u.combine('encoderType', kProgrammableEncoderTypes))
  .fn(t => {
    const { encoderType } = t.params;
    const bindGroup = t.createBindGroup('valid', 'buffer', encoderType, []);
    const dynamicOffsets = [0];

    const { encoder, validateFinish } = t.createEncoder(encoderType);
    encoder.setBindGroup(0, bindGroup, dynamicOffsets);
    validateFinish(false);
  });

g.test('dynamic_offsets_match_expectations_in_pass_encoder')
  .desc('Tests that given dynamicOffsets match the specified bindGroup.')
  .params(u =>
    u
      .combine('encoderType', kProgrammableEncoderTypes)
      .combineWithParams([
        { dynamicOffsets: [256, 0], _success: true }, // Dynamic offsets aligned
        { dynamicOffsets: [1, 2], _success: false }, // Dynamic offsets not aligned

        // Wrong number of dynamic offsets
        { dynamicOffsets: [256, 0, 0], _success: false },
        { dynamicOffsets: [256], _success: false },
        { dynamicOffsets: [], _success: false },

        // Dynamic uniform buffer out of bounds because of binding size
        { dynamicOffsets: [512, 0], _success: false },
        { dynamicOffsets: [1024, 0], _success: false },
        { dynamicOffsets: [0xffffffff, 0], _success: false },

        // Dynamic storage buffer out of bounds because of binding size
        { dynamicOffsets: [0, 512], _success: false },
        { dynamicOffsets: [0, 1024], _success: false },
        { dynamicOffsets: [0, 0xffffffff], _success: false },
      ])
      .combine('useU32array', [false, true])
  )
  .fn(t => {
    const kBindingSize = 12;

    const bindGroupLayout = t.device.createBindGroupLayout({
      entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.COMPUTE | GPUShaderStage.FRAGMENT,
          buffer: {
            type: 'uniform',
            hasDynamicOffset: true,
          },
        },
        {
          binding: 1,
          visibility: GPUShaderStage.COMPUTE | GPUShaderStage.FRAGMENT,
          buffer: {
            type: 'storage',
            hasDynamicOffset: true,
          },
        },
      ],
    });

    const uniformBuffer = t.device.createBuffer({
      size: 2 * kMinDynamicBufferOffsetAlignment + 8,
      usage: GPUBufferUsage.UNIFORM,
    });

    const storageBuffer = t.device.createBuffer({
      size: 2 * kMinDynamicBufferOffsetAlignment + 8,
      usage: GPUBufferUsage.STORAGE,
    });

    const bindGroup = t.device.createBindGroup({
      layout: bindGroupLayout,
      entries: [
        {
          binding: 0,
          resource: {
            buffer: uniformBuffer,
            size: kBindingSize,
          },
        },
        {
          binding: 1,
          resource: {
            buffer: storageBuffer,
            size: kBindingSize,
          },
        },
      ],
    });

    const { encoderType, dynamicOffsets, useU32array, _success } = t.params;

    const { encoder, validateFinish } = t.createEncoder(encoderType);
    if (useU32array) {
      encoder.setBindGroup(0, bindGroup, new Uint32Array(dynamicOffsets), 0, dynamicOffsets.length);
    } else {
      encoder.setBindGroup(0, bindGroup, dynamicOffsets);
    }
    validateFinish(_success);
  });

g.test('u32array_start_and_length')
  .desc('Tests that dynamicOffsetsData(Start|Length) apply to the given Uint32Array.')
  .paramsSubcasesOnly([
    // dynamicOffsetsDataLength > offsets.length
    {
      offsets: [0],
      dynamicOffsetsDataStart: 0,
      dynamicOffsetsDataLength: 2,
      _success: false,
    },
    // dynamicOffsetsDataStart + dynamicOffsetsDataLength > offsets.length
    {
      offsets: [0],
      dynamicOffsetsDataStart: 1,
      dynamicOffsetsDataLength: 1,
      _success: false,
    },
    {
      offsets: [0, 0],
      dynamicOffsetsDataStart: 1,
      dynamicOffsetsDataLength: 1,
      _success: true,
    },
    {
      offsets: [0, 0, 0],
      dynamicOffsetsDataStart: 1,
      dynamicOffsetsDataLength: 1,
      _success: true,
    },
    {
      offsets: [0, 0],
      dynamicOffsetsDataStart: 0,
      dynamicOffsetsDataLength: 2,
      _success: true,
    },
  ])
  .fn(t => {
    const { offsets, dynamicOffsetsDataStart, dynamicOffsetsDataLength, _success } = t.params;
    const kBindingSize = 8;

    const bindGroupLayout = t.device.createBindGroupLayout({
      entries: range(dynamicOffsetsDataLength, i => ({
        binding: i,
        visibility: GPUShaderStage.FRAGMENT,
        buffer: {
          type: 'storage',
          hasDynamicOffset: true,
        },
      })),
    });

    const bindGroup = t.device.createBindGroup({
      layout: bindGroupLayout,
      entries: range(dynamicOffsetsDataLength, i => ({
        binding: i,
        resource: {
          buffer: t.createBufferWithState('valid', {
            size: kBindingSize,
            usage: GPUBufferUsage.STORAGE,
          }),
          size: kBindingSize,
        },
      })),
    });

    const { encoder, validateFinish } = t.createEncoder('render pass');

    const doSetBindGroup = () => {
      encoder.setBindGroup(
        0,
        bindGroup,
        new Uint32Array(offsets),
        dynamicOffsetsDataStart,
        dynamicOffsetsDataLength
      );
    };

    if (_success) {
      doSetBindGroup();
    } else {
      t.shouldThrow('RangeError', doSetBindGroup);
    }

    // RangeError in setBindGroup does not cause the encoder to become invalid.
    validateFinish(true);
  });

g.test('buffer_dynamic_offsets')
  .desc(
    `
    Test that the dynamic offsets of the BufferLayout is a multiple of
    'minUniformBufferOffsetAlignment|minStorageBufferOffsetAlignment' if the BindGroup entry defines
    buffer and the buffer type is 'uniform|storage|read-only-storage'.
  `
  )
  .params(u =>
    u //
      .combine('type', kBufferBindingTypes)
      .combine('encoderType', kProgrammableEncoderTypes)
      .beginSubcases()
      .expand('dynamicOffset', ({ type }) =>
        type === 'uniform'
          ? [
              kLimitInfo.minUniformBufferOffsetAlignment.default,
              kLimitInfo.minUniformBufferOffsetAlignment.default * 0.5,
              kLimitInfo.minUniformBufferOffsetAlignment.default * 1.5,
              kLimitInfo.minUniformBufferOffsetAlignment.default * 2,
              kLimitInfo.minUniformBufferOffsetAlignment.default + 2,
            ]
          : [
              kLimitInfo.minStorageBufferOffsetAlignment.default,
              kLimitInfo.minStorageBufferOffsetAlignment.default * 0.5,
              kLimitInfo.minStorageBufferOffsetAlignment.default * 1.5,
              kLimitInfo.minStorageBufferOffsetAlignment.default * 2,
              kLimitInfo.minStorageBufferOffsetAlignment.default + 2,
            ]
      )
  )
  .fn(t => {
    const { type, dynamicOffset, encoderType } = t.params;
    const kBindingSize = 12;

    const bindGroupLayout = t.device.createBindGroupLayout({
      entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.COMPUTE,
          buffer: { type, hasDynamicOffset: true },
        },
      ],
    });

    let usage, isValid;
    if (type === 'uniform') {
      usage = GPUBufferUsage.UNIFORM;
      isValid = dynamicOffset % kLimitInfo.minUniformBufferOffsetAlignment.default === 0;
    } else {
      usage = GPUBufferUsage.STORAGE;
      isValid = dynamicOffset % kLimitInfo.minStorageBufferOffsetAlignment.default === 0;
    }

    const buffer = t.device.createBuffer({
      size: 3 * kMinDynamicBufferOffsetAlignment,
      usage,
    });

    const bindGroup = t.device.createBindGroup({
      entries: [{ binding: 0, resource: { buffer, size: kBindingSize } }],
      layout: bindGroupLayout,
    });

    const { encoder, validateFinish } = t.createEncoder(encoderType);
    encoder.setBindGroup(0, bindGroup, [dynamicOffset]);
    validateFinish(isValid);
  });
