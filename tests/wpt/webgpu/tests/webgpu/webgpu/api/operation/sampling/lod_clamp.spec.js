/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests the behavior of LOD clamping (lodMinClamp, lodMaxClamp).

TODO:
- Write a test that can test the exact clamping behavior
- Test a bunch of values, including very large/small ones.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);