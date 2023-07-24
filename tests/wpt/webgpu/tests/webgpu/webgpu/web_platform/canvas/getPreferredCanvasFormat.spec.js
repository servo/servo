/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests for navigator.gpu.getPreferredCanvasFormat.
`;
import { Fixture } from '../../../common/framework/fixture.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';

export const g = makeTestGroup(Fixture);

g.test('value')
  .desc(
    `
    Ensure getPreferredCanvasFormat returns one of the valid values.
    `
  )
  .fn(t => {
    const preferredFormat = navigator.gpu.getPreferredCanvasFormat();
    t.expect(preferredFormat === 'bgra8unorm' || preferredFormat === 'rgba8unorm');
  });
