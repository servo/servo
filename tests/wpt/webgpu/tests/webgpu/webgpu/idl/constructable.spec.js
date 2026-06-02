/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test that constructable WebGPU objects are actually constructable.
`;import { makeTestGroup } from './../../common/framework/test_group.js';
import { IDLTest } from './idl_test.js';

export const g = makeTestGroup(IDLTest);

g.test('gpu_errors').
desc('tests that GPUErrors are constructable').
params((u) =>
u.combine('errorType', [
'GPUInternalError',
'GPUOutOfMemoryError',
'GPUValidationError']
)
).
fn((t) => {
  const { errorType } = t.params;
  const Ctor = globalThis[errorType];
  const msg = 'this is a test';
  const error = new Ctor(msg);
  t.expect(error.message === msg);
});

const pipelineErrorOptions = [
{ reason: 'validation' },
{ reason: 'internal' }];


g.test('pipeline_errors').
desc('tests that GPUPipelineError is constructable').
params((u) =>
u //
.combine('msg', [undefined, 'some msg']).
combine('options', pipelineErrorOptions)
).
fn((t) => {
  const { msg, options } = t.params;
  const error = new GPUPipelineError(msg, options);
  const expectedMsg = msg || '';
  t.expect(error.message === expectedMsg);
  t.expect(error.reason === options.reason);
});

g.test('uncaptured_error_event').
desc('tests that GPUUncapturedErrorEvent is constructable').
fn((t) => {
  const msg = 'this is a test';
  const error = new GPUValidationError(msg);
  const event = new GPUUncapturedErrorEvent('uncapturedError', { error });
  t.expect(event.error === error);
});