// AUTO-GENERATED - DO NOT EDIT. See src/common/tools/gen_listings.ts.

export const listing = [
  {
    "file": [],
    "readme": "WebGPU conformance test suite."
  },
  {
    "file": [
      "api"
    ],
    "readme": "Tests for full coverage of the Javascript API surface of WebGPU."
  },
  {
    "file": [
      "api",
      "operation"
    ],
    "readme": "Tests that check the result of performing valid WebGPU operations, taking advantage of\nparameterization to exercise interactions between features."
  },
  {
    "file": [
      "api",
      "operation",
      "buffers"
    ],
    "readme": "GPUBuffer tests."
  },
  {
    "file": [
      "api",
      "operation",
      "buffers",
      "map"
    ],
    "description": ""
  },
  {
    "file": [
      "api",
      "operation",
      "buffers",
      "map_detach"
    ],
    "description": ""
  },
  {
    "file": [
      "api",
      "operation",
      "buffers",
      "map_oom"
    ],
    "description": ""
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "basic"
    ],
    "description": "Basic tests."
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "copies"
    ],
    "description": "copy{Buffer,Texture}To{Buffer,Texture} tests."
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "render",
      "basic"
    ],
    "description": "Basic command buffer rendering tests."
  },
  {
    "file": [
      "api",
      "operation",
      "copyBetweenLinearDataAndTexture"
    ],
    "description": "writeTexture + copyBufferToTexture + copyTextureToBuffer operation tests.\n\n* copy_with_various_rows_per_image_and_bytes_per_row: test that copying data with various bytesPerRow (including { ==, > } bytesInACompleteRow) and rowsPerImage (including { ==, > } copyExtent.height) values and minimum required bytes in copy works for every format. Also covers special code paths:\n  - bufferSize - offset < bytesPerImage * copyExtent.depth\n  - when bytesPerRow is not a multiple of 512 and copyExtent.depth > 1: copyExtent.depth % 2 == { 0, 1 }\n  - bytesPerRow == bytesInACompleteCopyImage\n\n* copy_with_various_offsets_and_data_sizes: test that copying data with various offset (including { ==, > } 0 and is/isn't power of 2) values and additional data paddings works for every format with 2d and 2d-array textures. Also covers special code paths:\n  - offset + bytesInCopyExtentPerRow { ==, > } bytesPerRow\n  - offset > bytesInACompleteCopyImage\n\n* copy_with_various_origins_and_copy_extents: test that copying slices of a texture works with various origin (including { origin.x, origin.y, origin.z } { ==, > } 0 and is/isn't power of 2) and copyExtent (including { copyExtent.x, copyExtent.y, copyExtent.z } { ==, > } 0 and is/isn't power of 2) values (also including {origin._ + copyExtent._ { ==, < } the subresource size of textureCopyView) works for all formats. origin and copyExtent values are passed as [number, number, number] instead of GPUExtent3DDict.\n\n* copy_various_mip_levels: test that copying various mip levels works for all formats. Also covers special code paths:\n  - the physical size of the subresouce is not equal to the logical size\n  - bufferSize - offset < bytesPerImage * copyExtent.depth and copyExtent needs to be clamped\n\n* copy_with_no_image_or_slice_padding_and_undefined_values: test that when copying a single row we can set any bytesPerRow value and when copying a single slice we can set rowsPerImage to 0. Also test setting offset, rowsPerImage, mipLevel, origin, origin.{x,y,z} to undefined.\n\n* TODO:\n  - add another initMethod which renders the texture\n  - because of expectContests 4-bytes alignment we don't test CopyT2B with buffer size not divisible by 4\n  - add tests for 1d / 3d textures"
  },
  {
    "file": [
      "api",
      "operation",
      "fences"
    ],
    "description": ""
  },
  {
    "file": [
      "api",
      "operation",
      "render_pass",
      "storeOp"
    ],
    "description": "API Operation Tests for RenderPass StoreOp.\n\n  Test Coverage:\n\n  - Tests that color and depth-stencil store operations {'clear', 'store'} work correctly for a\n    render pass with both a color attachment and depth-stencil attachment.\n      TODO: use depth24plus-stencil8\n\n  - Tests that store operations {'clear', 'store'} work correctly for a render pass with multiple\n    color attachments.\n      TODO: test with more interesting loadOp values\n\n  - Tests that store operations {'clear', 'store'} work correctly for a render pass with a color\n    attachment for:\n      - All renderable color formats\n      - mip level set to {'0', mip > '0'}\n      - array layer set to {'0', layer > '1'} for 2D textures\n      TODO: depth slice set to {'0', slice > '0'} for 3D textures\n\n  - Tests that store operations {'clear', 'store'} work correctly for a render pass with a\n    depth-stencil attachment for:\n      - All renderable depth-stencil formats\n      - mip level set to {'0', mip > '0'}\n      - array layer set to {'0', layer > '1'} for 2D textures\n      TODO: test depth24plus and depth24plus-stencil8 formats\n      TODO: test that depth and stencil aspects are set seperately\n      TODO: depth slice set to {'0', slice > '0'} for 3D textures\n      TODO: test with more interesting loadOp values"
  },
  {
    "file": [
      "api",
      "operation",
      "resource_init",
      "copied_texture_clear"
    ],
    "description": "Test uninitialized textures are initialized to zero when copied."
  },
  {
    "file": [
      "api",
      "regression"
    ],
    "readme": "One-off tests that reproduce API bugs found in implementations to prevent the bugs from\nappearing again."
  },
  {
    "file": [
      "api",
      "validation"
    ],
    "readme": "Positive and negative tests for all the validation rules of the API."
  },
  {
    "file": [
      "api",
      "validation",
      "copyBufferToBuffer"
    ],
    "description": "copyBufferToBuffer tests.\n\nTest Plan:\n* Buffer is valid/invalid\n  - the source buffer is invalid\n  - the destination buffer is invalid\n* Buffer usages\n  - the source buffer is created without GPUBufferUsage::COPY_SRC\n  - the destination buffer is created without GPUBufferUsage::COPY_DEST\n* CopySize\n  - copySize is not a multiple of 4\n  - copySize is 0\n* copy offsets\n  - sourceOffset is not a multiple of 4\n  - destinationOffset is not a multiple of 4\n* Arthimetic overflow\n  - (sourceOffset + copySize) is overflow\n  - (destinationOffset + copySize) is overflow\n* Out of bounds\n  - (sourceOffset + copySize) > size of source buffer\n  - (destinationOffset + copySize) > size of destination buffer\n* Source buffer and destination buffer are the same buffer"
  },
  {
    "file": [
      "api",
      "validation",
      "copy_between_linear_data_and_texture"
    ],
    "readme": "writeTexture + copyBufferToTexture + copyTextureToBuffer validation tests.\n\nTest coverage:\n* resource usages:\n\t- texture_usage_must_be_valid: for GPUTextureUsage::COPY_SRC, GPUTextureUsage::COPY_DST flags.\n\t- TODO: buffer_usage_must_be_valid\n\n* textureCopyView:\n\t- texture_must_be_valid: for valid, destroyed, error textures.\n\t- sample_count_must_be_1: for sample count 1 and 4.\n\t- mip_level_must_be_in_range: for various combinations of mipLevel and mipLevelCount.\n\t- texel_block_alignment_on_origin: for all formats and coordinates.\n\n* bufferCopyView:\n\t- TODO: buffer_must_be_valid\n\t- TODO: bytes_per_row_alignment\n\n* linear texture data:\n\t- bound_on_rows_per_image: for various combinations of copyDepth (1, >1), copyHeight, rowsPerImage.\n\t- offset_plus_required_bytes_in_copy_overflow\n\t- required_bytes_in_copy: testing minimal data size and data size too small for various combinations of bytesPerRow, rowsPerImage, copyExtent and offset. for the copy method, bytesPerRow is computed as bytesInACompleteRow aligned to be a multiple of 256 + bytesPerRowPadding * 256.\n\t- texel_block_alignment_on_rows_per_image: for all formats.\n\t- texel_block_alignment_on_offset: for all formats.\n\t- bound_on_bytes_per_row: for all formats and various combinations of bytesPerRow and copyExtent. for writeTexture, bytesPerRow is computed as (blocksPerRow * blockWidth * bytesPerBlock + additionalBytesPerRow) and copyExtent.width is computed as copyWidthInBlocks * blockWidth. for the copy methods, both values are mutliplied by 256.\n\t- bound_on_offset: for various combinations of offset and dataSize.\n\n* texture copy range:\n\t- 1d_texture: copyExtent.height isn't 1, copyExtent.depth isn't 1.\n\t- texel_block_alignment_on_size: for all formats and coordinates.\n\t- texture_range_conditons: for all coordinate and various combinations of origin, copyExtent, textureSize and mipLevel.\n\nTODO: more test coverage for 1D and 3D textures."
  },
  {
    "file": [
      "api",
      "validation",
      "copy_between_linear_data_and_texture",
      "copyBetweenLinearDataAndTexture_dataRelated"
    ],
    "description": ""
  },
  {
    "file": [
      "api",
      "validation",
      "copy_between_linear_data_and_texture",
      "copyBetweenLinearDataAndTexture_textureRelated"
    ],
    "description": ""
  },
  {
    "file": [
      "api",
      "validation",
      "createBindGroup"
    ],
    "description": "createBindGroup validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "createBindGroupLayout"
    ],
    "description": "createBindGroupLayout validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "createPipelineLayout"
    ],
    "description": "createPipelineLayout validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "createTexture"
    ],
    "description": "createTexture validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "createView"
    ],
    "description": "createView validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "error_scope"
    ],
    "description": "error scope validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "fences"
    ],
    "description": "fences validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "queue_submit"
    ],
    "description": "queue submit validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "render_pass",
      "resolve"
    ],
    "description": "API Validation Tests for RenderPass Resolve.\n\n  Test Coverage:\n    - When resolveTarget is not null:\n      - Test that the colorAttachment is multisampled:\n        - A single sampled colorAttachment should generate an error.\n      - Test that the resolveTarget is single sampled:\n        - A multisampled resolveTarget should generate an error.\n      - Test that the resolveTarget has usage OUTPUT_ATTACHMENT:\n        - A resolveTarget without usage OUTPUT_ATTACHMENT should generate an error.\n      - Test that the resolveTarget's texture view describes a single subresource:\n        - A resolveTarget texture view with base mip {0, base mip > 0} and mip count of 1 should be\n          valid.\n          - An error should be generated when the resolve target view mip count is not 1 and base\n            mip is {0, base mip > 0}.\n        - A resolveTarget texture view with base array layer {0, base array layer > 0} and array\n          layer count of 1 should be valid.\n          - An error should be generated when the resolve target view array layer count is not 1 and\n            base array layer is {0, base array layer > 0}.\n      - Test that the resolveTarget's format is the same as the colorAttachment:\n        - An error should be generated when the resolveTarget's format does not match the\n          colorAttachment's format.\n      - Test that the resolveTarget's size is the same the colorAttachment:\n        - An error should be generated when the resolveTarget's height or width are not equal to\n          the colorAttachment's height or width."
  },
  {
    "file": [
      "api",
      "validation",
      "render_pass",
      "storeOp"
    ],
    "description": "API Validation Tests for RenderPass StoreOp.\n\nTest Coverage:\n  - Tests that when depthReadOnly is true, depthStoreOp must be 'store'.\n    - When depthReadOnly is true and depthStoreOp is 'clear', an error should be generated.\n\n  - Tests that when stencilReadOnly is true, stencilStoreOp must be 'store'.\n    - When stencilReadOnly is true and stencilStoreOp is 'clear', an error should be generated.\n\n  - Tests that the depthReadOnly value matches the stencilReadOnly value.\n    - When depthReadOnly does not match stencilReadOnly, an error should be generated.\n\n  - Tests that depthReadOnly and stencilReadOnly default to false."
  },
  {
    "file": [
      "api",
      "validation",
      "render_pass_descriptor"
    ],
    "description": "render pass descriptor validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "resource_usages",
      "textureUsageInRender"
    ],
    "description": "Texture Usages Validation Tests in Render Pass.\n\nTest Coverage:\n\n  - For each combination of two texture usages:\n    - For various subresource ranges (different mip levels or array layers) that overlap a given\n      subresources or not for color formats:\n      - Check that an error is generated when read-write or write-write usages are binding to the\n        same texture subresource. Otherwise, no error should be generated. One exception is race\n        condition upon two writeonly-storage-texture usages, which is valid.\n\n  - For each combination of two texture usages:\n    - For various aspects (all, depth-only, stencil-only) that overlap a given subresources or not\n      for depth/stencil formats:\n      - Check that an error is generated when read-write or write-write usages are binding to the\n        same aspect. Otherwise, no error should be generated.\n\n  - Test combinations of two shader stages:\n    - Texture usages in bindings with invisible shader stages should be tracked. Invisible shader\n      stages include shader stage with visibility none and compute shader stage in render pass.\n\n  - Tests replaced bindings:\n    - Texture usages via bindings replaced by another setBindGroup() upon the same bindGroup index\n      in current scope should be tracked.\n\n  - Test texture usages in bundle:\n    - Texture usages in bundle should be tracked if that bundle is executed in the current scope."
  },
  {
    "file": [
      "api",
      "validation",
      "setBindGroup"
    ],
    "description": "setBindGroup validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "setBlendColor"
    ],
    "description": "setBlendColor validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "setScissorRect"
    ],
    "description": "setScissorRect validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "setStencilReference"
    ],
    "description": "setStencilReference validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "setViewport"
    ],
    "description": "setViewport validation tests."
  },
  {
    "file": [
      "examples"
    ],
    "description": "Examples of writing CTS tests with various features.\n\nStart here when looking for examples of basic framework usage."
  },
  {
    "file": [
      "idl"
    ],
    "readme": "Tests to check that the WebGPU IDL is correctly implemented, for examples that objects exposed\nexactly the correct members, and that methods throw when passed incomplete dictionaries."
  },
  {
    "file": [
      "idl",
      "constants",
      "flags"
    ],
    "description": "Test the values of flags interfaces (e.g. GPUTextureUsage)."
  },
  {
    "file": [
      "shader"
    ],
    "readme": "Tests for full coverage of the shaders that can be passed to WebGPU."
  },
  {
    "file": [
      "shader",
      "execution"
    ],
    "readme": "Tests that check the result of valid shader execution."
  },
  {
    "file": [
      "shader",
      "regression"
    ],
    "readme": "One-off tests that reproduce shader bugs found in implementations to prevent the bugs from\nappearing again."
  },
  {
    "file": [
      "shader",
      "validation"
    ],
    "readme": "Positive and negative tests for all the validation rules of the shading language."
  },
  {
    "file": [
      "web-platform"
    ],
    "readme": "Tests for Web platform-specific interactions like GPUSwapChain and canvas, WebXR,\nImageBitmaps, and video APIs."
  },
  {
    "file": [
      "web-platform",
      "canvas",
      "context_creation"
    ],
    "description": ""
  },
  {
    "file": [
      "web-platform",
      "copyImageBitmapToTexture"
    ],
    "description": "copy imageBitmap To texture tests."
  }
];
