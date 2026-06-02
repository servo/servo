/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for canvas context creation.

Note there are no context creation attributes for WebGPU (as of this writing).
Options are configured in configure() instead.
`;import { Fixture } from '../../../common/framework/fixture.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';

export const g = makeTestGroup(Fixture);

g.test('return_type').
desc(
  `Test the return type of getContext for WebGPU.

    TODO: Test OffscreenCanvas made from transferControlToOffscreen.`
).
params((u) =>
u //
.combine('offscreen', [false, true]).
beginSubcases().
combine('attributes', [undefined, {}])
).
fn((t) => {
  let canvas;
  if (t.params.offscreen) {
    if (typeof OffscreenCanvas === 'undefined') {
      // Skip if the current context doesn't have OffscreenCanvas (e.g. Node).
      t.skip('OffscreenCanvas is not available in this context');
    }

    canvas = new OffscreenCanvas(10, 10);
  } else {
    if (typeof document === 'undefined') {
      // Skip if there is no document (Workers, Node)
      t.skip('DOM is not available to create canvas element');
    }

    canvas = document.createElement('canvas', t.params.attributes);
    canvas.width = 10;
    canvas.height = 10;
  }

  const ctx = canvas.getContext('webgpu');
  t.expect(ctx instanceof GPUCanvasContext);
});