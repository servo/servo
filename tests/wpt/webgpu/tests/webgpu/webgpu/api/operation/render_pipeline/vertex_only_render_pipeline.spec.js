/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Test vertex-only render pipeline.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

class F extends GPUTest {}

export const g = makeTestGroup(F);

g.test('draw_depth_and_stencil_with_vertex_only_pipeline')
  .desc(
    `
TODO:
- Test drawing depth and stencil with vertex-only render pipelines by
  1. Create a color attachment and depth-stencil attachment of 4 pixels in a line, clear the color
     to RGBA(0.0, 0.0, 0.0, 0.0), depth to 0.0 and stencil to 0x0
  2. Use a depth and stencil test disabled vertex-only render pipeline to modify the depth of middle
     2 pixels to 0.5, while leaving stencil unchanged
  3. Use another depth and stencil test disabled vertex-only render pipeline to modify the stencil
     of right 2 pixels to 0x1, while leaving depth unchanged
  4. Use a complete render pipeline to draw all 4 pixels with color RGBA(0.0, 1.0, 0.0, 1.0), but
     with depth test requiring depth no less than 0.5 and stencil test requiring stencil equals to 0x1
  5. Validate that only the third pixel is of color RGBA(0.0, 1.0, 0.0, 1.0), and all other pixels
     are RGBA(0.0, 0.0, 0.0, 0.0).
`
  )
  .unimplemented();
