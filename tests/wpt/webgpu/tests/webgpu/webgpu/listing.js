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
    "description": "Texture Usages Validation Tests in Render Pass.\n\nTest Coverage:\n - Tests that read and write usages upon the same texture subresource, or different subresources\n   of the same texture. Different subresources of the same texture includes different mip levels,\n   different array layers, and different aspects.\n   - When read and write usages are binding to the same texture subresource, an error should be\n     generated. Otherwise, no error should be generated."
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
