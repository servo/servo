/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for default pipeline layouts.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('getBindGroupLayout_js_object').
desc(
  `Test that getBindGroupLayout returns [TODO: the same or a different, needs spec] object
each time.`
).
unimplemented();

g.test('incompatible_with_explicit').
desc(`Test that default bind group layouts are never compatible with explicitly created ones.`).
unimplemented();

g.test('layout').
desc(
  `Test that bind group layouts of the default pipeline layout are correct by passing various
shaders and then checking their computed bind group layouts are compatible with particular bind
groups.`
).
unimplemented();