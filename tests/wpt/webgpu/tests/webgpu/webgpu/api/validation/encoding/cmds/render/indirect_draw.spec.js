/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Validation tests for drawIndirect/drawIndexedIndirect on render pass and render bundle.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUConst } from '../../../../../constants.js';
import { kResourceStates } from '../../../../../gpu_test.js';
import { ValidationTest } from '../../../validation_test.js';

import { kRenderEncodeTypeParams } from './render.js';

const kIndirectDrawTestParams = kRenderEncodeTypeParams.combine('indexed', [true, false]);

class F extends ValidationTest {
  makeIndexBuffer() {
    return this.device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.INDEX,
    });
  }
}

export const g = makeTestGroup(F);

g.test('indirect_buffer_state')
  .desc(
    `
Tests indirect buffer must be valid.
  `
  )
  .paramsSubcasesOnly(kIndirectDrawTestParams.combine('state', kResourceStates))
  .fn(t => {
    const { encoderType, indexed, state } = t.params;
    const pipeline = t.createNoOpRenderPipeline();
    const indirectBuffer = t.createBufferWithState(state, {
      size: 256,
      usage: GPUBufferUsage.INDIRECT,
    });

    const { encoder, validateFinishAndSubmitGivenState } = t.createEncoder(encoderType);
    encoder.setPipeline(pipeline);
    if (indexed) {
      const indexBuffer = t.makeIndexBuffer();
      encoder.setIndexBuffer(indexBuffer, 'uint32');
      encoder.drawIndexedIndirect(indirectBuffer, 0);
    } else {
      encoder.drawIndirect(indirectBuffer, 0);
    }

    validateFinishAndSubmitGivenState(state);
  });

g.test('indirect_buffer,device_mismatch')
  .desc(
    'Tests draw(Indexed)Indirect cannot be called with an indirect buffer created from another device'
  )
  .paramsSubcasesOnly(kIndirectDrawTestParams.combine('mismatched', [true, false]))
  .beforeAllSubcases(t => {
    t.selectMismatchedDeviceOrSkipTestCase(undefined);
  })
  .fn(t => {
    const { encoderType, indexed, mismatched } = t.params;

    const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

    const indirectBuffer = sourceDevice.createBuffer({
      size: 256,
      usage: GPUBufferUsage.INDIRECT,
    });
    t.trackForCleanup(indirectBuffer);

    const { encoder, validateFinish } = t.createEncoder(encoderType);
    encoder.setPipeline(t.createNoOpRenderPipeline());

    if (indexed) {
      encoder.setIndexBuffer(t.makeIndexBuffer(), 'uint32');
      encoder.drawIndexedIndirect(indirectBuffer, 0);
    } else {
      encoder.drawIndirect(indirectBuffer, 0);
    }
    validateFinish(!mismatched);
  });

g.test('indirect_buffer_usage')
  .desc(
    `
Tests indirect buffer must have 'Indirect' usage.
  `
  )
  .paramsSubcasesOnly(
    kIndirectDrawTestParams.combine('usage', [
      GPUConst.BufferUsage.INDIRECT, // control case
      GPUConst.BufferUsage.COPY_DST,
      GPUConst.BufferUsage.COPY_DST | GPUConst.BufferUsage.INDIRECT,
    ])
  )
  .fn(t => {
    const { encoderType, indexed, usage } = t.params;
    const indirectBuffer = t.device.createBuffer({
      size: 256,
      usage,
    });

    const { encoder, validateFinish } = t.createEncoder(encoderType);
    encoder.setPipeline(t.createNoOpRenderPipeline());
    if (indexed) {
      const indexBuffer = t.makeIndexBuffer();
      encoder.setIndexBuffer(indexBuffer, 'uint32');
      encoder.drawIndexedIndirect(indirectBuffer, 0);
    } else {
      encoder.drawIndirect(indirectBuffer, 0);
    }
    validateFinish((usage & GPUBufferUsage.INDIRECT) !== 0);
  });

g.test('indirect_offset_alignment')
  .desc(
    `
Tests indirect offset must be a multiple of 4.
  `
  )
  .paramsSubcasesOnly(kIndirectDrawTestParams.combine('indirectOffset', [0, 2, 4]))
  .fn(t => {
    const { encoderType, indexed, indirectOffset } = t.params;
    const pipeline = t.createNoOpRenderPipeline();
    const indirectBuffer = t.device.createBuffer({
      size: 256,
      usage: GPUBufferUsage.INDIRECT,
    });

    const { encoder, validateFinish } = t.createEncoder(encoderType);
    encoder.setPipeline(pipeline);
    if (indexed) {
      const indexBuffer = t.makeIndexBuffer();
      encoder.setIndexBuffer(indexBuffer, 'uint32');
      encoder.drawIndexedIndirect(indirectBuffer, indirectOffset);
    } else {
      encoder.drawIndirect(indirectBuffer, indirectOffset);
    }

    validateFinish(indirectOffset % 4 === 0);
  });

g.test('indirect_offset_oob')
  .desc(
    `
Tests indirect draw calls with various indirect offsets and buffer sizes.
- (offset, b.size) is
  - (0, 0)
  - (0, min size) (control case)
  - (0, min size + 1) (control case)
  - (0, min size - 1)
  - (0, min size - min alignment)
  - (min alignment, min size + min alignment)
  - (min alignment, min size + min alignment - 1)
  - (min alignment / 2, min size + min alignment)
  - (min alignment +/- 1, min size + min alignment)
  - (min size, min size)
  - (min size + min alignment, min size)
  - min size = indirect draw parameters size
  - x =(drawIndirect, drawIndexedIndirect)
  `
  )
  .paramsSubcasesOnly(
    kIndirectDrawTestParams.expandWithParams(p => {
      const indirectParamsSize = p.indexed ? 20 : 16;
      return [
        { indirectOffset: 0, bufferSize: 0, _valid: false },
        { indirectOffset: 0, bufferSize: indirectParamsSize, _valid: true },
        { indirectOffset: 0, bufferSize: indirectParamsSize + 1, _valid: true },
        { indirectOffset: 0, bufferSize: indirectParamsSize - 1, _valid: false },
        { indirectOffset: 0, bufferSize: indirectParamsSize - 4, _valid: false },
        { indirectOffset: 4, bufferSize: indirectParamsSize + 4, _valid: true },
        { indirectOffset: 4, bufferSize: indirectParamsSize + 3, _valid: false },
        { indirectOffset: 2, bufferSize: indirectParamsSize + 4, _valid: false },
        { indirectOffset: 3, bufferSize: indirectParamsSize + 4, _valid: false },
        { indirectOffset: 5, bufferSize: indirectParamsSize + 4, _valid: false },
        { indirectOffset: indirectParamsSize, bufferSize: indirectParamsSize, _valid: false },
        { indirectOffset: indirectParamsSize + 4, bufferSize: indirectParamsSize, _valid: false },
      ];
    })
  )
  .fn(t => {
    const { encoderType, indexed, indirectOffset, bufferSize, _valid } = t.params;
    const pipeline = t.createNoOpRenderPipeline();
    const indirectBuffer = t.device.createBuffer({
      size: bufferSize,
      usage: GPUBufferUsage.INDIRECT,
    });

    const { encoder, validateFinish } = t.createEncoder(encoderType);
    encoder.setPipeline(pipeline);
    if (indexed) {
      const indexBuffer = t.makeIndexBuffer();
      encoder.setIndexBuffer(indexBuffer, 'uint32');
      encoder.drawIndexedIndirect(indirectBuffer, indirectOffset);
    } else {
      encoder.drawIndirect(indirectBuffer, indirectOffset);
    }

    validateFinish(_valid);
  });
