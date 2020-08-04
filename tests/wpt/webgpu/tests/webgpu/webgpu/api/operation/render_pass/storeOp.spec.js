/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `API Operation Tests for RenderPass StoreOp.

  Test Coverage Needed:

  - Test that a render pass has correct output for combinations of:
    - All color attachments from '0' to 'MAX_COLOR_ATTACHMENTS' with combinations of:
      - storeOp set to {'clear', 'store', 'undefined}
      - All color renderable formats
      - mip level set to {'0', mip > '0'}
      - array layer set to {'0', layer > '1'} for 2D textures
      - depth slice set to {'0', slice > '0'} for 3D textures
    - With and without a depthStencilAttachment that has the combinations of:
      - depthStoreOp set to {'clear', 'store', 'undefined'}
      - stencilStoreOp set to {'clear', 'store', 'undefined'}
      - All depth/stencil formats
      - mip level set to {'0', mip > '0'}
      - array layer set to {'0', layer > '1'} for 2D textures
      - depth slice set to {'0', slice > '0'} for 3D textures`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);
