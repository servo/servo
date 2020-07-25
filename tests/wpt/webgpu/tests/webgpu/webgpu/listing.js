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
      "create_mapped"
    ],
    "description": ""
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
      "compute",
      "basic"
    ],
    "description": "Basic command buffer compute tests."
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
      "command_buffer",
      "render",
      "rendering"
    ],
    "description": ""
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "render",
      "storeop"
    ],
    "description": "renderPass store op test that drawn quad is either stored or cleared based on storeop"
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
    "description": "API Operation Tests for RenderPass StoreOp.\n\n  Test Coverage Needed:\n\n  - Test that a render pass has correct output for combinations of:\n    - All color attachments from '0' to 'MAX_COLOR_ATTACHMENTS' with combinations of:\n      - storeOp set to {'clear', 'store', 'undefined}\n      - All color renderable formats\n      - mip level set to {'0', mip > '0'}\n      - array layer set to {'0', layer > '1'} for 2D textures\n      - depth slice set to {'0', slice > '0'} for 3D textures\n    - With and without a depthStencilAttachment that has the combinations of:\n      - depthStoreOp set to {'clear', 'store', 'undefined'}\n      - stencilStoreOp set to {'clear', 'store', 'undefined'}\n      - All depth/stencil formats\n      - mip level set to {'0', mip > '0'}\n      - array layer set to {'0', layer > '1'} for 2D textures\n      - depth slice set to {'0', slice > '0'} for 3D textures"
  },
  {
    "file": [
      "api",
      "operation",
      "render_pipeline",
      "culling_tests"
    ],
    "description": "Test culling and rasterizaion state.\n\nTest coverage:\nTest all culling combinations of GPUFrontFace and GPUCullMode show the correct output.\n\nUse 2 triangles with different winding orders:\n\n- Test that the counter-clock wise triangle has correct output for:\n  - All FrontFaces (ccw, cw)\n  - All CullModes (none, front, back)\n  - All depth stencil attachment types (none, depth24plus, depth32float, depth24plus-stencil8)\n  - Some primitive topologies (triangle-list, TODO: triangle-strip)\n\n- Test that the clock wise triangle has correct output for:\n  - All FrontFaces (ccw, cw)\n  - All CullModes (none, front, back)\n  - All depth stencil attachment types (none, depth24plus, depth32float, depth24plus-stencil8)\n  - Some primitive topologies (triangle-list, TODO: triangle-strip)"
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
      "operation",
      "resource_init",
      "depth_stencil_attachment_clear"
    ],
    "description": "Test uninitialized textures are initialized to zero when used as a depth/stencil attachment."
  },
  {
    "file": [
      "api",
      "operation",
      "resource_init",
      "sampled_texture_clear"
    ],
    "description": "Test uninitialized textures are initialized to zero when sampled."
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
      "createRenderPipeline"
    ],
    "description": "createRenderPipeline validation tests."
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
      "encoding",
      "cmds",
      "index_access"
    ],
    "description": "indexed draws validation tests."
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
      "render_pass"
    ],
    "description": "render pass validation tests."
  },
  {
    "file": [
      "api",
      "validation",
      "render_pass",
      "storeOp"
    ],
    "description": "API Validation Tests for RenderPass StoreOp.\n\n  Test Coverage Needed:\n\n  - Test that when depthReadOnly is true, depthStoreOp must be 'store'\n\n  - Test that when stencilReadOnly is true, stencilStoreOp must be 'store'"
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
      "setVertexBuffer"
    ],
    "description": "setVertexBuffer validation tests."
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
      "api",
      "validation",
      "vertex_state"
    ],
    "description": "vertexState validation tests."
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
      "execution",
      "robust_access"
    ],
    "description": "Tests to check array clamping in shaders is correctly implemented including vector / matrix indexing"
  },
  {
    "file": [
      "shader",
      "execution",
      "robust_access_vertex"
    ],
    "description": "Test vertex attributes behave correctly (no crash / data leak) when accessed out of bounds\n\nTest coverage:\n\nThe following will be parameterized (all combinations tested):\n\n1) Draw call indexed? (false / true)\n  - Run the draw call using an index buffer\n\n2) Draw call indirect? (false / true)\n  - Run the draw call using an indirect buffer\n\n3) Draw call parameter (vertexCount, firstVertex, indexCount, firstIndex, baseVertex, instanceCount,\n  firstInstance)\n  - The parameter which will go out of bounds. Filtered depending on if the draw call is indexed.\n\n4) Attribute type (float, vec2, vec3, vec4)\n  - The input attribute type in the vertex shader\n\n5) Error scale (1, 4, 10^2, 10^4, 10^6)\n  - Offset to add to the correct draw call parameter\n\n6) Additional vertex buffers (0, +4)\n  - Tests that no OOB occurs if more vertex buffers are used\n\nThe tests will also have another vertex buffer bound for an instanced attribute, to make sure\ninstanceCount / firstInstance are tested.\n\nThe tests will include multiple attributes per vertex buffer.\n\nThe vertex buffers will be filled by repeating a few chosen values until the end of the buffer.\n\nThe test will run a render pipeline which verifies the following:\n1) All vertex attribute values occur in the buffer or are zero\n2) All gl_VertexIndex values are within the index buffer or 0\n\nTODO:\n\nA suppression may be needed for d3d12 on tests that have non-zero baseVertex, since d3d12 counts\nfrom 0 instead of from baseVertex (will fail check for gl_VertexIndex).\n\nVertex buffer contents could be randomized to prevent the case where a previous test creates\na similar buffer to ours and the OOB-read seems valid. This should be deterministic, which adds\nmore complexity that we may not need."
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
