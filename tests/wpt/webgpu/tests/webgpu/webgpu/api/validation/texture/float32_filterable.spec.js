/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for capabilities added by float32-filterable flag.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kTextureSampleTypes } from '../../../capability_info.js';
import { ValidationTest } from '../validation_test.js';

export const g = makeTestGroup(ValidationTest);

const kFloat32Formats = ['r32float', 'rg32float', 'rgba32float'];

g.test('create_bind_group').
desc(
  `
Test that it is valid to bind a float32 texture format to a 'float' sampled texture iff
float32-filterable is enabled.
`
).
params((u) =>
u.
combine('enabled', [true, false]).
beginSubcases().
combine('format', kFloat32Formats).
combine('sampleType', kTextureSampleTypes)
).
beforeAllSubcases((t) => {
  if (t.params.enabled) {
    t.selectDeviceOrSkipTestCase('float32-filterable');
  }
}).
fn((t) => {
  const { enabled, format, sampleType } = t.params;
  const layout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      texture: { sampleType }
    }]

  });
  const textureDesc = {
    size: { width: 4, height: 4 },
    format,
    usage: GPUTextureUsage.TEXTURE_BINDING
  };
  const shouldError = !(
  enabled && sampleType === 'float' ||
  sampleType === 'unfilterable-float');

  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource: t.device.createTexture(textureDesc).createView() }],
      layout
    });
  }, shouldError);
});