/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
API validation test for debug groups and markers

Test Coverage:
  - For each encoder type (GPUCommandEncoder, GPUComputeEncoder, GPURenderPassEncoder,
  GPURenderBundleEncoder):
    - Test that all pushDebugGroup must have a corresponding popDebugGroup
      - Push and pop counts of 0, 1, and 2 will be used.
      - An error must be generated for non matching counts.
    - Test calling pushDebugGroup with empty and non-empty strings.
    - Test inserting a debug marker with empty and non-empty strings.
    - Test strings with \0 in them.
    - Test non-ASCII strings.
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { kEncoderTypes } from '../../../../util/command_buffer_maker.js';
import { ValidationTest } from '../../validation_test.js';

export const g = makeTestGroup(ValidationTest);

g.test('debug_group_balanced')
  .params(u =>
    u
      .combine('encoderType', kEncoderTypes)
      .beginSubcases()
      .combine('pushCount', [0, 1, 2])
      .combine('popCount', [0, 1, 2])
  )
  .fn(t => {
    const { encoder, validateFinishAndSubmit } = t.createEncoder(t.params.encoderType);
    for (let i = 0; i < t.params.pushCount; ++i) {
      encoder.pushDebugGroup(`${i}`);
    }
    for (let i = 0; i < t.params.popCount; ++i) {
      encoder.popDebugGroup();
    }
    validateFinishAndSubmit(t.params.pushCount === t.params.popCount, true);
  });

g.test('debug_group')
  .params(u =>
    u //
      .combine('encoderType', kEncoderTypes)
      .beginSubcases()
      .combine('label', ['', 'group', 'null\0in\0group\0label', '\0null at beginning', '🌞👆'])
  )
  .fn(t => {
    const { encoder, validateFinishAndSubmit } = t.createEncoder(t.params.encoderType);
    encoder.pushDebugGroup(t.params.label);
    encoder.popDebugGroup();
    validateFinishAndSubmit(true, true);
  });

g.test('debug_marker')
  .params(u =>
    u //
      .combine('encoderType', kEncoderTypes)
      .beginSubcases()
      .combine('label', ['', 'marker', 'null\0in\0marker', '\0null at beginning', '🌞👆'])
  )
  .fn(t => {
    const { encoder, validateFinishAndSubmit } = t.createEncoder(t.params.encoderType);
    encoder.insertDebugMarker(t.params.label);
    validateFinishAndSubmit(true, true);
  });
