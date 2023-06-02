/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests the behavior of different filtering modes in minFilter/magFilter/mipmapFilter.

TODO:
- Test exact sampling results with small tolerance. Tests should differentiate between different
  values for all three filter modes to make sure none are missed or incorrect in implementations.
- (Likely unnecessary with the above.) Test exactly the expected number of samples are used.
  Test this by setting up a rendering and asserting how many different shades result.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);
