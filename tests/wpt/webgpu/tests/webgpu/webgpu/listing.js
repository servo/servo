// AUTO-GENERATED - DO NOT EDIT. See src/common/tools/gen_listings_and_webworkers.ts.

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
      "adapter",
      "info"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "adapter",
      "requestAdapter"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "adapter",
      "requestDevice"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "async_ordering"
    ],
    "readme": "Test ordering of async resolutions between promises returned by the following calls (and possibly\nbetween multiple of the same call), where there are constraints on the ordering.\nSpec issue: https://github.com/gpuweb/gpuweb/issues/962\n\nTODO: plan and implement\n- createReadyPipeline() (not sure if this actually has any ordering constraints)\n- cmdbuf.executionTime\n- device.popErrorScope()\n- device.lost\n- queue.onSubmittedWorkDone()\n- buffer.mapAsync()\n- shadermodule.getCompilationInfo()"
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
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "buffers",
      "map_ArrayBuffer"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "buffers",
      "map_detach"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "buffers",
      "map_oom"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "buffers",
      "threading"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "basic"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "clearBuffer"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "copyBufferToBuffer"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "copyTextureToTexture"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "image_copy"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "programmable",
      "state_tracking"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "queries"
    ],
    "readme": "TODO: test the behavior of creating/using/resolving queries.\n- timestamp\n- nested (e.g. timestamp inside occlusion query), if any such cases are valid. Try\n  writing to the same query set (at same or different indices), if valid. Check results make sense.\n- start a query (all types) with no draw calls"
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "queries",
      "occlusionQuery"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "render",
      "dynamic_state"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "command_buffer",
      "render",
      "state_tracking"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "compute",
      "basic"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "compute_pipeline",
      "entry_point_name"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "compute_pipeline",
      "overrides"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "device",
      "lost"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "labels"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "memory_allocation"
    ],
    "readme": "Try to stress memory allocators in the implementation and driver.\n\nTODO: plan and implement\n- Tests which (pseudo-randomly?) allocate a bunch of memory and then assert things about the memory\n  (it's not aliased, it's valid to read and write in various ways, accesses read/write the correct data)\n    - Possibly also with OOB accesses/robust buffer access?\n- Tests which are targeted against particular known implementation details"
  },
  {
    "file": [
      "api",
      "operation",
      "memory_sync",
      "buffer",
      "multiple_buffers"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "memory_sync",
      "buffer",
      "single_buffer"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "memory_sync",
      "texture",
      "readonly_depth_stencil"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "memory_sync",
      "texture",
      "same_subresource"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "onSubmittedWorkDone"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "pipeline",
      "default_layout"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "queue",
      "writeBuffer"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "reflection"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "render_pass"
    ],
    "readme": "Render pass stuff other than commands (which are in command_buffer/)."
  },
  {
    "file": [
      "api",
      "operation",
      "render_pass",
      "clear_value"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "render_pass",
      "resolve"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "render_pass",
      "storeOp"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "render_pass",
      "storeop2"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "render_pipeline",
      "culling_tests"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "render_pipeline",
      "overrides"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "render_pipeline",
      "pipeline_output_targets"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "render_pipeline",
      "primitive_topology"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "render_pipeline",
      "sample_mask"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "render_pipeline",
      "vertex_only_render_pipeline"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "rendering",
      "3d_texture_slices"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "rendering",
      "basic"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "rendering",
      "color_target_state"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "rendering",
      "depth"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "rendering",
      "depth_bias"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "rendering",
      "depth_clip_clamp"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "rendering",
      "draw"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "rendering",
      "indirect_draw"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "rendering",
      "robust_access_index"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "rendering",
      "stencil"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "resource_init",
      "buffer"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "resource_init",
      "texture_zero"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "sampling",
      "anisotropy"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "sampling",
      "filter_mode"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "sampling",
      "lod_clamp"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "shader_module",
      "compilation_info"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "storage_texture",
      "read_only"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "storage_texture",
      "read_write"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "texture_view",
      "format_reinterpretation"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "texture_view",
      "read"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "texture_view",
      "write"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "threading"
    ],
    "readme": "Tests for behavior with multiple threads (main thread + workers).\n\nTODO: plan and implement\n- 'postMessage'\n  Try postMessage'ing an object of every type (to same or different thread)\n    - {main -> main, main -> worker, worker -> main, worker1 -> worker1, worker1 -> worker2}\n    - through {global postMessage, MessageChannel}\n    - {in, not in} transferrable object list, when valid\n- 'concurrency'\n  Short tight loop doing many of an action from two threads at the same time\n    - e.g. {create {buffer, texture, shader, pipeline}}"
  },
  {
    "file": [
      "api",
      "operation",
      "uncapturederror"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "vertex_state",
      "correctness"
    ]
  },
  {
    "file": [
      "api",
      "operation",
      "vertex_state",
      "index_format"
    ]
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
      "buffer",
      "create"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "buffer",
      "destroy"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "buffer",
      "mapping"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "buffer",
      "threading"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "features"
    ],
    "readme": "Test every method or option that shouldn't be allowed without a feature enabled.\nIf the feature is not enabled, any use of an enum value added by a feature must be an\n*exception*, per <https://github.com/gpuweb/gpuweb/blob/main/design/ErrorConventions.md>.\n\n- x= that feature {enabled, disabled}\n\nGenerally one file for each feature name, but some may be grouped (e.g. one file for all optional\nquery types, one file for all optional texture formats).\n\nTODO: implement"
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "features",
      "clip_distances"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "features",
      "query_types"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "features",
      "texture_formats"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits"
    ],
    "readme": "Test everything that shouldn't be valid without a higher-than-specified limit.\n\n- x= that limit {default, max supported (if different), lower than default (TODO: if allowed)}\n\nOne file for each limit name.\n\nTODO: implement\nTODO: Also test that \"alignment\" limits require a power of 2."
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxBindGroups"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxBindGroupsPlusVertexBuffers"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxBindingsPerBindGroup"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxBufferSize"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxColorAttachmentBytesPerSample"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxColorAttachments"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxComputeInvocationsPerWorkgroup"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxComputeWorkgroupSizeX"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxComputeWorkgroupSizeY"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxComputeWorkgroupSizeZ"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxComputeWorkgroupStorageSize"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxComputeWorkgroupsPerDimension"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxDynamicStorageBuffersPerPipelineLayout"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxDynamicUniformBuffersPerPipelineLayout"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxInterStageShaderVariables"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxSampledTexturesPerShaderStage"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxSamplersPerShaderStage"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxStorageBufferBindingSize"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxStorageBuffersPerShaderStage"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxStorageTexturesPerShaderStage"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxTextureArrayLayers"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxTextureDimension1D"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxTextureDimension2D"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxTextureDimension3D"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxUniformBufferBindingSize"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxUniformBuffersPerShaderStage"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxVertexAttributes"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxVertexBufferArrayStride"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "maxVertexBuffers"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "minStorageBufferOffsetAlignment"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "capability_checks",
      "limits",
      "minUniformBufferOffsetAlignment"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "compute_pipeline"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "createBindGroup"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "createBindGroupLayout"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "createPipelineLayout"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "createSampler"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "createTexture"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "createView"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "debugMarker"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "beginComputePass"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "beginRenderPass"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "clearBuffer"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "compute_pass"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "copyBufferToBuffer"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "copyTextureToTexture"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "debug"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "index_access"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "render",
      "draw"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "render",
      "dynamic_state"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "render",
      "indirect_draw"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "render",
      "setIndexBuffer"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "render",
      "setPipeline"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "render",
      "setVertexBuffer"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "render",
      "state_tracking"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "render_pass"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "cmds",
      "setBindGroup"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "createRenderBundleEncoder"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "encoder_open_state"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "encoder_state"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "programmable",
      "pipeline_bind_group_compat"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "queries",
      "begin_end"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "queries",
      "general"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "queries",
      "resolveQuerySet"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "encoding",
      "render_bundle"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "error_scope"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "getBindGroupLayout"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "gpu_external_texture_expiration"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "image_copy"
    ],
    "readme": "writeTexture + copyBufferToTexture + copyTextureToBuffer validation tests.\n\nTest coverage:\n* resource usages:\n  - texture_usage_must_be_valid: for GPUTextureUsage::COPY_SRC, GPUTextureUsage::COPY_DST flags.\n  - buffer_usage_must_be_valid: for GPUBufferUsage::COPY_SRC, GPUBufferUsage::COPY_DST flags.\n\n* textureCopyView:\n  - texture_must_be_valid: for valid, destroyed, error textures.\n  - sample_count_must_be_1: for sample count 1 and 4.\n  - mip_level_must_be_in_range: for various combinations of mipLevel and mipLevelCount.\n  - format: for all formats with full and non-full copies on width, height, and depth.\n  - texel_block_alignment_on_origin: for all formats and coordinates.\n\n* bufferCopyView:\n  - buffer_must_be_valid: for valid, destroyed, error buffers.\n  - bytes_per_row_alignment: for bytesPerRow to be 256-byte aligned or not, and bytesPerRow is required or not.\n\n* linear texture data:\n  - bound_on_rows_per_image: for various combinations of copyDepth (1, >1), copyHeight, rowsPerImage.\n  - offset_plus_required_bytes_in_copy_overflow\n  - required_bytes_in_copy: testing minimal data size and data size too small for various combinations of bytesPerRow, rowsPerImage, copyExtent and offset. for the copy method, bytesPerRow is computed as bytesInACompleteRow aligned to be a multiple of 256 + bytesPerRowPadding * 256.\n  - texel_block_alignment_on_rows_per_image: for all formats.\n  - offset_alignment: for all formats.\n  - bound_on_offset: for various combinations of offset and dataSize.\n\n* texture copy range:\n  - 1d_texture: copyExtent.height isn't 1, copyExtent.depthOrArrayLayers isn't 1.\n  - texel_block_alignment_on_size: for all formats and coordinates.\n  - texture_range_conditons: for all coordinate and various combinations of origin, copyExtent, textureSize and mipLevel.\n\nTODO: more test coverage for 1D and 3D textures."
  },
  {
    "file": [
      "api",
      "validation",
      "image_copy",
      "buffer_related"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "image_copy",
      "buffer_texture_copies"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "image_copy",
      "layout_related"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "image_copy",
      "texture_related"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "layout_shader_compat"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "query_set",
      "create"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "query_set",
      "destroy"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "queue"
    ],
    "readme": "Tests for validation that occurs inside queued operations\n(submit, writeBuffer, writeTexture, copyExternalImageToTexture).\n\nBufferMapStatesToTest = {\n  mapped -> unmapped,\n  mapped at creation -> unmapped,\n  mapping pending -> unmapped,\n  pending -> mapped (await map),\n  unmapped -> pending (noawait map),\n  created mapped-at-creation,\n}\n\nNote writeTexture is tested in image_copy."
  },
  {
    "file": [
      "api",
      "validation",
      "queue",
      "buffer_mapped"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "queue",
      "copyToTexture",
      "CopyExternalImageToTexture"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "queue",
      "destroyed",
      "buffer"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "queue",
      "destroyed",
      "query_set"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "queue",
      "destroyed",
      "texture"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "queue",
      "submit"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "queue",
      "writeBuffer"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "queue",
      "writeTexture"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pass"
    ],
    "readme": "Render pass stuff other than commands (which are in encoding/cmds/)."
  },
  {
    "file": [
      "api",
      "validation",
      "render_pass",
      "attachment_compatibility"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pass",
      "render_pass_descriptor"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pass",
      "resolve"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pipeline",
      "depth_stencil_state"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pipeline",
      "float32_blendable"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pipeline",
      "fragment_state"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pipeline",
      "inter_stage"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pipeline",
      "misc"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pipeline",
      "multisample_state"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pipeline",
      "overrides"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pipeline",
      "primitive_state"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pipeline",
      "resource_compatibility"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pipeline",
      "shader_module"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "render_pipeline",
      "vertex_state"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "resource_usages",
      "buffer"
    ],
    "readme": "TODO: look at texture,*"
  },
  {
    "file": [
      "api",
      "validation",
      "resource_usages",
      "buffer",
      "in_pass_encoder"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "resource_usages",
      "buffer",
      "in_pass_misc"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "resource_usages",
      "texture",
      "in_pass_encoder"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "resource_usages",
      "texture",
      "in_render_common"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "resource_usages",
      "texture",
      "in_render_misc"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "shader_module",
      "entry_point"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "shader_module",
      "overrides"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "state",
      "device_lost"
    ],
    "readme": "Tests of behavior while the device is lost.\n\n- x= every method in the API.\n\nTODO: implement"
  },
  {
    "file": [
      "api",
      "validation",
      "state",
      "device_lost",
      "destroy"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "texture",
      "bgra8unorm_storage"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "texture",
      "destroy"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "texture",
      "float32_filterable"
    ]
  },
  {
    "file": [
      "api",
      "validation",
      "texture",
      "rg11b10ufloat_renderable"
    ]
  },
  {
    "file": [
      "compat",
      "api",
      "validation",
      "createBindGroup"
    ]
  },
  {
    "file": [
      "compat",
      "api",
      "validation",
      "createBindGroupLayout"
    ]
  },
  {
    "file": [
      "compat",
      "api",
      "validation",
      "encoding",
      "cmds",
      "copyTextureToBuffer"
    ]
  },
  {
    "file": [
      "compat",
      "api",
      "validation",
      "encoding",
      "cmds",
      "copyTextureToTexture"
    ]
  },
  {
    "file": [
      "compat",
      "api",
      "validation",
      "encoding",
      "programmable",
      "pipeline_bind_group_compat"
    ]
  },
  {
    "file": [
      "compat",
      "api",
      "validation",
      "render_pipeline",
      "depth_stencil_state"
    ]
  },
  {
    "file": [
      "compat",
      "api",
      "validation",
      "render_pipeline",
      "fragment_state"
    ]
  },
  {
    "file": [
      "compat",
      "api",
      "validation",
      "render_pipeline",
      "unsupported_wgsl"
    ]
  },
  {
    "file": [
      "compat",
      "api",
      "validation",
      "render_pipeline",
      "vertex_state"
    ]
  },
  {
    "file": [
      "compat",
      "api",
      "validation",
      "texture",
      "createTexture"
    ]
  },
  {
    "file": [
      "compat",
      "api",
      "validation",
      "texture",
      "cubeArray"
    ]
  },
  {
    "file": [
      "examples"
    ]
  },
  {
    "file": [
      "idl"
    ],
    "readme": "Tests to check that the WebGPU IDL is correctly implemented, for examples that objects exposed\nexactly the correct members, and that methods throw when passed incomplete dictionaries.\n\nSee https://github.com/gpuweb/cts/issues/332\n\nTODO: exposed.html.ts: Test all WebGPU interfaces instead of just some of them.\nTODO: Check prototype chains. (Add a helper in IDLTest for this.)"
  },
  {
    "file": [
      "idl",
      "constants",
      "flags"
    ]
  },
  {
    "file": [
      "idl",
      "constructable"
    ]
  },
  {
    "file": [
      "print_environment"
    ]
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
      "expression",
      "access",
      "array",
      "index"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "access",
      "matrix",
      "index"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "access",
      "structure",
      "index"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "access",
      "vector",
      "components"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "access",
      "vector",
      "index"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "af_addition"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "af_comparison"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "af_division"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "af_matrix_addition"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "af_matrix_matrix_multiplication"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "af_matrix_scalar_multiplication"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "af_matrix_subtraction"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "af_matrix_vector_multiplication"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "af_multiplication"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "af_remainder"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "af_subtraction"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "ai_arithmetic"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "ai_comparison"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "bitwise"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "bitwise_shift"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "bool_logical"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f16_addition"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f16_comparison"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f16_division"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f16_matrix_addition"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f16_matrix_matrix_multiplication"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f16_matrix_scalar_multiplication"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f16_matrix_subtraction"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f16_matrix_vector_multiplication"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f16_multiplication"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f16_remainder"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f16_subtraction"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f32_addition"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f32_comparison"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f32_division"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f32_matrix_addition"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f32_matrix_matrix_multiplication"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f32_matrix_scalar_multiplication"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f32_matrix_subtraction"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f32_matrix_vector_multiplication"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f32_multiplication"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f32_remainder"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "f32_subtraction"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "i32_arithmetic"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "i32_comparison"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "u32_arithmetic"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "binary",
      "u32_comparison"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "abs"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "acos"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "acosh"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "all"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "any"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "arrayLength"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "asin"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "asinh"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atan"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atan2"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atanh"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atomics",
      "atomicAdd"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atomics",
      "atomicAnd"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atomics",
      "atomicCompareExchangeWeak"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atomics",
      "atomicExchange"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atomics",
      "atomicLoad"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atomics",
      "atomicMax"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atomics",
      "atomicMin"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atomics",
      "atomicOr"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atomics",
      "atomicStore"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atomics",
      "atomicSub"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "atomics",
      "atomicXor"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "bitcast"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "ceil"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "clamp"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "cos"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "cosh"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "countLeadingZeros"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "countOneBits"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "countTrailingZeros"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "cross"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "degrees"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "determinant"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "distance"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "dot"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "dot4I8Packed"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "dot4U8Packed"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "dpdx"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "dpdxCoarse"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "dpdxFine"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "dpdy"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "dpdyCoarse"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "dpdyFine"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "exp"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "exp2"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "extractBits"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "faceForward"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "firstLeadingBit"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "firstTrailingBit"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "floor"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "fma"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "fract"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "frexp"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "fwidth"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "fwidthCoarse"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "fwidthFine"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "insertBits"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "inversesqrt"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "ldexp"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "length"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "log"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "log2"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "max"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "min"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "mix"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "modf"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "normalize"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "pack2x16float"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "pack2x16snorm"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "pack2x16unorm"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "pack4x8snorm"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "pack4x8unorm"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "pack4xI8"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "pack4xI8Clamp"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "pack4xU8"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "pack4xU8Clamp"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "pow"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "quadBroadcast"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "quadSwap"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "quantizeToF16"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "radians"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "reflect"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "refract"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "reverseBits"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "round"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "saturate"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "select"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "sign"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "sin"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "sinh"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "smoothstep"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "sqrt"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "step"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "storageBarrier"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "subgroupAdd"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "subgroupAll"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "subgroupAny"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "subgroupBallot"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "subgroupBitwise"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "subgroupBroadcast"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "subgroupMul"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "tan"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "tanh"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureDimensions"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureGather"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureGatherCompare"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureLoad"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureNumLayers"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureNumLevels"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureNumSamples"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureSample"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureSampleBaseClampToEdge"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureSampleBias"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureSampleCompare"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureSampleCompareLevel"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureSampleGrad"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureSampleLevel"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "textureStore"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "transpose"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "trunc"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "unpack2x16float"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "unpack2x16snorm"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "unpack2x16unorm"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "unpack4x8snorm"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "unpack4x8unorm"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "unpack4xI8"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "unpack4xU8"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "workgroupBarrier"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "builtin",
      "workgroupUniformLoad"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "call",
      "user",
      "ptr_params"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "constructor",
      "non_zero"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "constructor",
      "zero_value"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "precedence"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "address_of_and_indirection"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "af_arithmetic"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "af_assignment"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "ai_arithmetic"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "ai_assignment"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "ai_complement"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "bool_conversion"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "bool_logical"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "f16_arithmetic"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "f16_conversion"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "f32_arithmetic"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "f32_conversion"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "i32_arithmetic"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "i32_complement"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "i32_conversion"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "u32_complement"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "expression",
      "unary",
      "u32_conversion"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "float_parse"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "flow_control",
      "call"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "flow_control",
      "complex"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "flow_control",
      "eval_order"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "flow_control",
      "for"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "flow_control",
      "if"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "flow_control",
      "loop"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "flow_control",
      "phony"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "flow_control",
      "return"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "flow_control",
      "switch"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "flow_control",
      "while"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "limits"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "memory_layout"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "memory_model",
      "adjacent"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "memory_model",
      "atomicity"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "memory_model",
      "barrier"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "memory_model",
      "coherence"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "memory_model",
      "texture_intra_invocation_coherence"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "memory_model",
      "weak"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "padding"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "robust_access"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "robust_access_vertex"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "shader_io",
      "compute_builtins"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "shader_io",
      "fragment_builtins"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "shader_io",
      "shared_structs"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "shader_io",
      "user_io"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "shader_io",
      "vertex_builtins"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "shader_io",
      "workgroup_size"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "shadow"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "stage"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "statement",
      "compound"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "statement",
      "discard"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "statement",
      "increment_decrement"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "statement",
      "phony"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "value_init"
    ]
  },
  {
    "file": [
      "shader",
      "execution",
      "zero_init"
    ]
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
      "shader",
      "validation",
      "const_assert",
      "const_assert"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "decl",
      "compound_statement"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "decl",
      "const"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "decl",
      "context_dependent_resolution"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "decl",
      "let"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "decl",
      "override"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "decl",
      "var"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "access",
      "array"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "access",
      "matrix"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "access",
      "structure"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "access",
      "vector"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "binary",
      "add_sub_mul"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "binary",
      "and_or_xor"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "binary",
      "bitwise_shift"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "binary",
      "comparison"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "binary",
      "div_rem"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "binary",
      "parse"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "binary",
      "short_circuiting_and_or"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "abs"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "acos"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "acosh"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "all"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "any"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "arrayLength"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "asin"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "asinh"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "atan"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "atan2"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "atanh"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "atomics"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "barriers"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "bitcast"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "ceil"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "clamp"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "cos"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "cosh"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "countLeadingZeros"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "countOneBits"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "countTrailingZeros"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "cross"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "degrees"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "derivatives"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "determinant"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "distance"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "dot"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "dot4I8Packed"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "dot4U8Packed"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "exp"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "exp2"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "extractBits"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "faceForward"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "firstLeadingBit"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "firstTrailingBit"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "floor"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "fma"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "fract"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "frexp"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "insertBits"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "inverseSqrt"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "ldexp"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "length"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "log"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "log2"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "max"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "min"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "mix"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "modf"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "normalize"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "pack2x16float"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "pack2x16snorm"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "pack2x16unorm"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "pack4x8snorm"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "pack4x8unorm"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "pack4xI8"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "pack4xI8Clamp"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "pack4xU8"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "pack4xU8Clamp"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "pow"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "quadBroadcast"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "quadSwap"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "quantizeToF16"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "radians"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "reflect"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "refract"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "reverseBits"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "round"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "saturate"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "select"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "sign"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "sin"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "sinh"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "smoothstep"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "sqrt"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "step"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "subgroupAdd"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "subgroupAnyAll"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "subgroupBallot"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "subgroupBitwise"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "subgroupBroadcast"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "subgroupBroadcastFirst"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "subgroupElect"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "subgroupMinMax"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "subgroupMul"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "subgroupShuffle"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "tan"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "tanh"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureDimensions"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureGather"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureGatherCompare"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureLoad"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureNumLayers"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureNumLevels"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureNumSamples"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureSample"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureSampleBaseClampToEdge"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureSampleBias"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureSampleCompare"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureSampleCompareLevel"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureSampleGrad"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureSampleLevel"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "textureStore"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "transpose"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "trunc"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "unpack2x16float"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "unpack2x16snorm"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "unpack2x16unorm"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "unpack4x8snorm"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "unpack4x8unorm"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "unpack4xI8"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "unpack4xU8"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "value_constructor"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "call",
      "builtin",
      "workgroupUniformLoad"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "early_evaluation"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "matrix",
      "add_sub"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "matrix",
      "and_or_xor"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "matrix",
      "bitwise_shift"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "matrix",
      "comparison"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "matrix",
      "div_rem"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "matrix",
      "mul"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "overload_resolution"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "precedence"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "unary",
      "address_of_and_indirection"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "unary",
      "arithmetic_negation"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "unary",
      "bitwise_complement"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "expression",
      "unary",
      "logical_negation"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "extension",
      "clip_distances"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "extension",
      "dual_source_blending"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "extension",
      "pointer_composite_access"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "extension",
      "readonly_and_readwrite_storage_textures"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "functions",
      "alias_analysis"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "functions",
      "restrictions"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "parse",
      "attribute"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "parse",
      "blankspace"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "parse",
      "comments"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "parse",
      "diagnostic"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "parse",
      "enable"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "parse",
      "identifiers"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "parse",
      "literal"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "parse",
      "must_use"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "parse",
      "requires"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "parse",
      "semicolon"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "parse",
      "shadow_builtins"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "parse",
      "source"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "align"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "binding"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "builtins"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "entry_point"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "group"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "group_and_binding"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "id"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "interpolate"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "invariant"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "layout_constraints"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "locations"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "pipeline_stage"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "size"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "shader_io",
      "workgroup_size"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "break"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "break_if"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "compound"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "const_assert"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "continue"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "continuing"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "discard"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "for"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "if"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "increment_decrement"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "loop"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "phony"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "return"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "statement_behavior"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "switch"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "statement",
      "while"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "types",
      "alias"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "types",
      "array"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "types",
      "atomics"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "types",
      "enumerant"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "types",
      "matrix"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "types",
      "pointer"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "types",
      "ref"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "types",
      "struct"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "types",
      "textures"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "types",
      "vector"
    ]
  },
  {
    "file": [
      "shader",
      "validation",
      "uniformity",
      "uniformity"
    ]
  },
  {
    "file": [
      "util",
      "texture",
      "color_space_conversions"
    ]
  },
  {
    "file": [
      "util",
      "texture",
      "texel_data"
    ]
  },
  {
    "file": [
      "util",
      "texture",
      "texture_ok"
    ]
  },
  {
    "file": [
      "web_platform"
    ],
    "readme": "Tests for Web platform-specific interactions like GPUCanvasContext and canvas, WebXR,\nImageBitmaps, and video APIs.\n\nTODO(#922): Also hopefully tests for user-initiated readbacks from WebGPU canvases\n(printing, save image as, etc.)"
  },
  {
    "file": [
      "web_platform",
      "canvas"
    ],
    "readme": "Tests for WebGPU <canvas> and OffscreenCanvas presentation."
  },
  {
    "file": [
      "web_platform",
      "canvas",
      "configure"
    ]
  },
  {
    "file": [
      "web_platform",
      "canvas",
      "context_creation"
    ]
  },
  {
    "file": [
      "web_platform",
      "canvas",
      "getCurrentTexture"
    ]
  },
  {
    "file": [
      "web_platform",
      "canvas",
      "getPreferredCanvasFormat"
    ]
  },
  {
    "file": [
      "web_platform",
      "canvas",
      "readbackFromWebGPUCanvas"
    ]
  },
  {
    "file": [
      "web_platform",
      "copyToTexture",
      "ImageBitmap"
    ]
  },
  {
    "file": [
      "web_platform",
      "copyToTexture",
      "ImageData"
    ]
  },
  {
    "file": [
      "web_platform",
      "copyToTexture"
    ],
    "readme": "Tests for copyToTexture from all possible sources (video, canvas, ImageBitmap, ...)"
  },
  {
    "file": [
      "web_platform",
      "copyToTexture",
      "canvas"
    ]
  },
  {
    "file": [
      "web_platform",
      "copyToTexture",
      "image"
    ]
  },
  {
    "file": [
      "web_platform",
      "copyToTexture",
      "video"
    ]
  },
  {
    "file": [
      "web_platform",
      "external_texture"
    ],
    "readme": "Tests for external textures."
  },
  {
    "file": [
      "web_platform",
      "external_texture",
      "video"
    ]
  },
  {
    "file": [
      "web_platform",
      "reftests"
    ],
    "readme": "Reference tests (reftests) for WebGPU canvas presentation.\n\nThese render some contents to a canvas using WebGPU, and WPT compares the rendering result with\nthe \"reference\" versions (in `ref/`) which render with 2D canvas.\n\nThis tests things like:\n- The canvas has the correct orientation.\n- The canvas renders with the correct transfer function.\n- The canvas blends and interpolates in the correct color encoding.\n\nTODO(#918): Test all possible color spaces (once we have more than 1)\nTODO(#921): Why is there sometimes a difference of 1 (e.g. 3f vs 40) in canvas_size_different_with_back_buffer_size?\nAnd why does chromium's image_diff show diffs on other pixels that don't seem to have diffs?\nTODO(#1093): Test rgba16float values which are out of gamut of the canvas but under SDR luminance.\nTODO(#1093): Test rgba16float values which are above SDR luminance.\nTODO(#1116): Test canvas scaling.\nTODO: Test transferControlToOffscreen, used from {the same,another} thread"
  },
  {
    "file": [
      "web_platform",
      "worker",
      "worker"
    ]
  }
];
