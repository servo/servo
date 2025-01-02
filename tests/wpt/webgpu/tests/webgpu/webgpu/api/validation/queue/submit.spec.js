/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests submit validation.

Note: destroyed buffer/texture/querySet are tested in destroyed/. (unless it gets moved here)
Note: buffer map state is tested in ./buffer_mapped.spec.ts.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ValidationTest } from '../validation_test.js';






class F extends ValidationTest {
  createCommandBuffer(options = {}) {
    const device = options.device ?? this.device;

    let cb;

    this.expectValidationError(() => {
      const encoder = device.createCommandEncoder();
      if (options.valid === false) {
        // Popping a debug group when none are pushed results in an invalid command buffer.
        encoder.popDebugGroup();
      }
      cb = encoder.finish();
    }, options.valid === false);

    return cb;
  }
}

export const g = makeTestGroup(F);

g.test('command_buffer,device_mismatch').
desc(
  `
    Tests submit cannot be called with command buffers created from another device
    Test with two command buffers to make sure all command buffers can be validated:
    - cb0 and cb1 from same device
    - cb0 and cb1 from different device
    `
).
paramsSubcasesOnly([
{ cb0Mismatched: false, cb1Mismatched: false }, // control case
{ cb0Mismatched: true, cb1Mismatched: false },
{ cb0Mismatched: false, cb1Mismatched: true }]
).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { cb0Mismatched, cb1Mismatched } = t.params;
  const mismatched = cb0Mismatched || cb1Mismatched;

  const cb0 = t.createCommandBuffer({ device: cb0Mismatched ? t.mismatchedDevice : t.device });
  const cb1 = t.createCommandBuffer({ device: cb1Mismatched ? t.mismatchedDevice : t.device });

  t.expectValidationError(() => {
    t.device.queue.submit([cb0, cb1]);
  }, mismatched);
});

g.test('command_buffer,duplicate_buffers').
desc(
  `
    Tests submit cannot be called with the same command buffer listed multiple times:
    `
).
fn((t) => {
  const cb = t.createCommandBuffer();

  t.expectValidationError(() => {
    t.device.queue.submit([cb, cb]);
  }, true);
});

g.test('command_buffer,submit_invalidates').
desc(
  `
    Tests that calling submit invalidates the command buffers passed to it:
    `
).
fn((t) => {
  const cb = t.createCommandBuffer();

  // Initial submit of a valid command buffer should pass.
  t.device.queue.submit([cb]);

  // Subsequent submits of the same command buffer should fail.
  t.expectValidationError(() => {
    t.device.queue.submit([cb]);
  });
});

g.test('command_buffer,invalid_submit_invalidates').
desc(
  `
    Tests that calling submit invalidates all command buffers passed to it, even
    if they're part of an invalid submit.
    `
).
fn((t) => {
  const cb1 = t.createCommandBuffer();
  const cb1_invalid = t.createCommandBuffer({ valid: false });

  // Submit should fail because on of the command buffers is invalid
  t.expectValidationError(() => {
    t.device.queue.submit([cb1, cb1_invalid]);
  });

  // Subsequent submits of the previously valid command buffer should fail.
  t.expectValidationError(() => {
    t.device.queue.submit([cb1]);
  });

  // The order of the invalid and valid command buffers in the submit array should not matter.
  const cb2 = t.createCommandBuffer();
  const cb2_invalid = t.createCommandBuffer({ valid: false });

  t.expectValidationError(() => {
    t.device.queue.submit([cb2_invalid, cb2]);
  });
  t.expectValidationError(() => {
    t.device.queue.submit([cb2]);
  });
});