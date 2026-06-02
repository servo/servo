/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
TODO:
- createCommandEncoder
- non-pass command, or beginPass, during {render, compute} pass
- {before (control case), after} finish()
    - x= {finish(), ... all non-pass commands}
- {before (control case), after} end()
    - x= {render, compute} pass
    - x= {finish(), ... all relevant pass commands}
    - x= {
        - before endPass (control case)
        - after endPass (no pass open)
        - after endPass+beginPass (a new pass of the same type is open)
        - }
    - should make whole encoder invalid
- ?
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { objectEquals } from '../../../../common/util/util.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';

class F extends AllFeaturesMaxLimitsGPUTest {
  beginRenderPass(commandEncoder, view) {
    return commandEncoder.beginRenderPass({
      colorAttachments: [
      {
        view,
        clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
  }

  createAttachmentTextureView() {
    const texture = this.createTextureTracked({
      format: 'rgba8unorm',
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    });
    return texture.createView();
  }
}

export const g = makeTestGroup(F);

g.test('pass_end_invalid_order').
desc(
  `
  Test that beginning a {compute,render} pass before ending the previous {compute,render} pass
  causes an error.

  TODO(https://github.com/gpuweb/gpuweb/issues/5207): Resolve whether a validation error
  should be raised immediately if '!firstPassEnd && endPasses = [1, 0]'.
  `
).
params((u) =>
u.
combine('pass0Type', ['compute', 'render']).
combine('pass1Type', ['compute', 'render']).
beginSubcases().
combine('firstPassEnd', [true, false]).
combine('endPasses', [[], [0], [1], [0, 1], [1, 0]])
// Don't end the first pass multiple times (that generates a validation error but doesn't invalidate the encoder)
.unless((p) => p.firstPassEnd && p.endPasses.includes(0))
).
fn((t) => {
  const { pass0Type, pass1Type, firstPassEnd, endPasses } = t.params;

  const view = t.createAttachmentTextureView();
  const encoder = t.device.createCommandEncoder();

  const firstPass =
  pass0Type === 'compute' ? encoder.beginComputePass() : t.beginRenderPass(encoder, view);

  if (firstPassEnd) firstPass.end();

  // Begin a second pass before ending the previous pass.
  const secondPass =
  pass1Type === 'compute' ? encoder.beginComputePass() : t.beginRenderPass(encoder, view);

  const passes = [firstPass, secondPass];
  for (const index of endPasses) {
    passes[index].end();
  }

  // If {endPasses} is '[1]' and {firstPass} ends, it's a control case.
  const valid = firstPassEnd && objectEquals(endPasses, [1]);

  t.expectValidationError(() => {
    encoder.finish();
  }, !valid);
});

g.test('call_after_successful_finish').
desc(`Test that encoding command after a successful finish generates a validation error.`).
params((u) =>
u.
combine('callCmd', [
'beginComputePass',
'beginRenderPass',
'finishAndSubmitFirst',
'finishAndSubmitSecond',
'insertDebugMarker']
).
beginSubcases().
combine('prePassType', ['compute', 'render', 'no-op']).
combine('IsEncoderFinished', [false, true])
).
fn((t) => {
  const { prePassType, IsEncoderFinished, callCmd } = t.params;

  const view = t.createAttachmentTextureView();
  const encoder = t.device.createCommandEncoder();

  if (prePassType !== 'no-op') {
    const pass =
    prePassType === 'compute' ? encoder.beginComputePass() : t.beginRenderPass(encoder, view);
    pass.end();
  }

  let buffer;
  if (IsEncoderFinished) {
    buffer = encoder.finish();
  }

  switch (callCmd) {
    case 'beginComputePass':
      {
        let pass;
        t.expectValidationError(() => {
          pass = encoder.beginComputePass();
        }, IsEncoderFinished);
        t.expectValidationError(() => {
          pass.end();
        }, IsEncoderFinished);
        if (buffer) {
          t.device.queue.submit([buffer]);
        }
      }
      break;
    case 'beginRenderPass':
      {
        let pass;
        t.expectValidationError(() => {
          pass = t.beginRenderPass(encoder, view);
        }, IsEncoderFinished);
        t.expectValidationError(() => {
          pass.end();
        }, IsEncoderFinished);
        if (buffer) {
          t.device.queue.submit([buffer]);
        }
      }
      break;
    case 'finishAndSubmitFirst':
      t.expectValidationError(() => {
        encoder.finish();
      }, IsEncoderFinished);
      if (buffer) {
        t.device.queue.submit([buffer]);
      }
      break;
    case 'finishAndSubmitSecond':
      {
        let secondBuffer;
        t.expectValidationError(() => {
          secondBuffer = encoder.finish();
        }, IsEncoderFinished);
        t.expectValidationError(() => {
          t.device.queue.submit([secondBuffer]);
        }, IsEncoderFinished);
      }
      break;
    case 'insertDebugMarker':
      t.expectValidationError(() => {
        encoder.insertDebugMarker('');
      }, IsEncoderFinished);
      if (buffer) {
        t.device.queue.submit([buffer]);
      }
      break;
  }

  if (!IsEncoderFinished && !callCmd.startsWith('finish')) {
    encoder.finish();
  }
});

g.test('pass_end_none').
desc(
  `
  Test that finishing an encoder without ending a child {compute,render} pass generates a validation error.
  `
).
paramsSubcasesOnly((u) => u.combine('passType', ['compute', 'render']).combine('endCount', [0, 1])).
fn((t) => {
  const { passType, endCount } = t.params;

  const view = t.createAttachmentTextureView();
  const encoder = t.device.createCommandEncoder();

  const pass =
  passType === 'compute' ? encoder.beginComputePass() : t.beginRenderPass(encoder, view);

  for (let i = 0; i < endCount; ++i) {
    pass.end();
  }

  t.expectValidationError(() => {
    encoder.finish();
  }, endCount === 0);
});

g.test('pass_end_twice,basic').
desc(
  'Test that ending a {compute,render} pass twice generates a validation error. The parent encoder (command encoder) can be either locked or open.'
).
paramsSubcasesOnly((u) =>
u //
.combine('passType', ['compute', 'render'])
// Simply end twice, the parent encoder is open at that time. If the second pass end is in the middle of another pass, the parent encoder is locked. It should generate a validation error in either situation.
.combine('endTwice', [false, true]).
combine('secondEndInAnotherPass', [false, 'compute', 'render']).
filter((p) => p.endTwice || !p.secondEndInAnotherPass)
).
fn((t) => {
  const { passType, endTwice, secondEndInAnotherPass } = t.params;

  const view = t.createAttachmentTextureView();
  const encoder = t.device.createCommandEncoder();

  const pass =
  passType === 'compute' ? encoder.beginComputePass() : t.beginRenderPass(encoder, view);

  pass.end();

  if (secondEndInAnotherPass) {
    const pass1 =
    secondEndInAnotherPass === 'compute' ?
    encoder.beginComputePass() :
    t.beginRenderPass(encoder, view);

    t.expectValidationError(() => {
      pass.end();
    });

    pass1.end();
  } else {
    if (endTwice) {
      t.expectValidationError(() => {
        pass.end();
      });
    }
  }

  encoder.finish();
});

g.test('pass_end_twice,render_pass_invalid').
desc(
  'Test that ending a render pass twice generates a validation error even if the pass is invalid.'
).
paramsSubcasesOnly((u) => u.combine('endTwice', [false, true])).
fn((t) => {
  const { endTwice } = t.params;

  const encoder = t.device.createCommandEncoder();
  // Pass encoder creation will fail because both color and depth/stencil attachments are empty.
  const pass = encoder.beginRenderPass({
    colorAttachments: []
  });

  pass.end();

  if (endTwice) {
    t.expectValidationError(() => {
      pass.end();
    });
  }

  t.expectValidationError(() => {
    encoder.finish();
  });
});

g.test('pass_begin_invalid_encoder').
desc(
  `
  Test that {compute,render} passes can still be opened on an invalid encoder.
  `
).
params((u) =>
u.
combine('pass0Type', ['compute', 'render']).
combine('pass1Type', ['compute', 'render']).
beginSubcases().
combine('firstPassInvalid', [false, true])
).
fn((t) => {
  const { pass0Type, pass1Type, firstPassInvalid } = t.params;

  const view = t.createAttachmentTextureView();

  const encoder = t.device.createCommandEncoder();

  let firstPass;
  if (pass0Type === 'compute') {
    firstPass = encoder.beginComputePass();
  } else {
    firstPass = t.beginRenderPass(encoder, view);
  }

  if (firstPassInvalid) {
    // Popping an empty debug group stack invalidates the pass.
    firstPass.popDebugGroup();
  }

  // Ending an invalid pass invalidates the encoder
  firstPass.end();

  // Passes can still be opened on an invalid encoder
  const secondPass =
  pass1Type === 'compute' ? encoder.beginComputePass() : t.beginRenderPass(encoder, view);

  secondPass.end();

  t.expectValidationError(() => {
    encoder.finish();
  }, firstPassInvalid);
});