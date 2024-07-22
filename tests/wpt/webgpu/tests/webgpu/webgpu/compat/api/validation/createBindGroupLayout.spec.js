/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests that, in compat mode, you can not create a bind group layout with unsupported storage texture formats.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kCompatModeUnsupportedStorageTextureFormats } from '../../../format_info.js';
import { CompatibilityTest } from '../../compatibility_test.js';

export const g = makeTestGroup(CompatibilityTest);

g.test('unsupportedStorageTextureFormats').
desc(
  `
      Tests that, in compat mode, you can not create a bind group layout with unsupported storage texture formats.
    `
).
params((u) => u.combine('format', kCompatModeUnsupportedStorageTextureFormats)).
fn((t) => {
  const { format } = t.params;

  t.expectValidationErrorInCompatibilityMode(() => {
    t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        storageTexture: {
          format
        }
      }]

    });
  }, true);
});