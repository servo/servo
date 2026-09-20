use webgpu_traits::id::{
    CommandEncoderId, ComputePassEncoderId, RenderBundleEncoderId, RenderPassEncoderId,
};
use webgpu_traits::{
    BindingCommand, CommandEncoderCommand, ComputePassEncoderCommand, DebugCommand,
    RenderBundleEncoderCommand, RenderCommand, RenderPassEncoderCommand,
};
use wgpu_core::command::{EncoderStateError, PassStateError};
use wgpu_core::global::Global;

pub(crate) fn handle_render_pass_command(
    global: &Global,
    render_pass_id: RenderPassEncoderId,
    command: RenderPassEncoderCommand,
) -> Result<(), PassStateError> {
    match command {
        RenderPassEncoderCommand::SetViewport {
            x,
            y,
            width,
            height,
            min_depth,
            max_depth,
        } => global.render_pass_set_viewport_with_id(
            render_pass_id,
            x,
            y,
            width,
            height,
            min_depth,
            max_depth,
        ),
        RenderPassEncoderCommand::SetScissorRect {
            x,
            y,
            width,
            height,
        } => global.render_pass_set_scissor_rect_with_id(render_pass_id, x, y, width, height),
        RenderPassEncoderCommand::SetBlendConstant(color) => {
            global.render_pass_set_blend_constant_with_id(render_pass_id, color)
        },
        RenderPassEncoderCommand::SetStencilReference(value) => {
            global.render_pass_set_stencil_reference_with_id(render_pass_id, value)
        },
        RenderPassEncoderCommand::BeginOcclusionQuery(query_index) => {
            global.render_pass_begin_occlusion_query_with_id(render_pass_id, query_index)
        },
        RenderPassEncoderCommand::EndOcclusionQuery => {
            global.render_pass_end_occlusion_query_with_id(render_pass_id)
        },
        RenderPassEncoderCommand::ExecuteBundles(ids) => {
            global.render_pass_execute_bundles_with_id(render_pass_id, &ids)
        },
        RenderPassEncoderCommand::BindingCommand(binding_command) => match binding_command {
            BindingCommand::SetBindGroup {
                index,
                bind_group,
                dynamic_offsets,
            } => global.render_pass_set_bind_group_with_id(
                render_pass_id,
                index,
                bind_group,
                &dynamic_offsets,
            ),
            BindingCommand::SetImmediates { range_offset, data } => {
                global.render_pass_set_immediates_with_id(render_pass_id, range_offset, &data)
            },
        },
        RenderPassEncoderCommand::RenderCommand(render_command) => match render_command {
            RenderCommand::SetPipeline(id) => {
                global.render_pass_set_pipeline_with_id(render_pass_id, id)
            },
            RenderCommand::SetIndexBuffer {
                buffer,
                index_format,
                offset,
                size,
            } => global.render_pass_set_index_buffer_with_id(
                render_pass_id,
                buffer,
                index_format,
                offset,
                size,
            ),
            RenderCommand::SetVertexBuffer {
                slot,
                buffer,
                offset,
                size,
            } => global.render_pass_set_vertex_buffer_with_id(
                render_pass_id,
                slot,
                buffer,
                offset,
                size,
            ),
            RenderCommand::Draw {
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            } => global.render_pass_draw_with_id(
                render_pass_id,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            ),
            RenderCommand::DrawIndexed {
                index_count,
                instance_count,
                first_index,
                base_vertex,
                first_instance,
            } => global.render_pass_draw_indexed_with_id(
                render_pass_id,
                index_count,
                instance_count,
                first_index,
                base_vertex,
                first_instance,
            ),
            RenderCommand::DrawIndirect {
                indirect_buffer,
                indirect_offset,
            } => global.render_pass_draw_indirect_with_id(
                render_pass_id,
                indirect_buffer,
                indirect_offset,
            ),
            RenderCommand::DrawIndexedIndirect {
                indirect_buffer,
                indirect_offset,
            } => global.render_pass_draw_indexed_indirect_with_id(
                render_pass_id,
                indirect_buffer,
                indirect_offset,
            ),
        },
        RenderPassEncoderCommand::DebugCommand(debug_command) => match debug_command {
            DebugCommand::PushDebugGroup(label) => {
                global.render_pass_push_debug_group_with_id(render_pass_id, &label, 0)
            },
            DebugCommand::PopDebugGroup => {
                global.render_pass_pop_debug_group_with_id(render_pass_id)
            },
            DebugCommand::InsertDebugMarker(label) => {
                global.render_pass_insert_debug_marker_with_id(render_pass_id, &label, 0)
            },
        },
    }
}

pub(crate) fn handle_render_bundle_command(
    global: &Global,
    render_bundle_encoder_id: RenderBundleEncoderId,
    command: RenderBundleEncoderCommand,
) -> Result<(), PassStateError> {
    match command {
        RenderBundleEncoderCommand::BindingCommand(binding_command) => match binding_command {
            BindingCommand::SetBindGroup {
                index,
                bind_group,
                dynamic_offsets,
            } => global.render_bundle_encoder_set_bind_group_with_id(
                render_bundle_encoder_id,
                index,
                bind_group,
                &dynamic_offsets,
            ),
            BindingCommand::SetImmediates { range_offset, data } => global
                .render_bundle_encoder_set_immediates_with_id(
                    render_bundle_encoder_id,
                    range_offset,
                    &data,
                ),
        },
        RenderBundleEncoderCommand::RenderCommand(render_command) => match render_command {
            RenderCommand::SetPipeline(id) => {
                global.render_bundle_encoder_set_pipeline_with_id(render_bundle_encoder_id, id)
            },
            RenderCommand::SetIndexBuffer {
                buffer,
                index_format,
                offset,
                size,
            } => global.render_bundle_encoder_set_index_buffer_with_id(
                render_bundle_encoder_id,
                buffer,
                index_format,
                offset,
                size,
            ),
            RenderCommand::SetVertexBuffer {
                slot,
                buffer,
                offset,
                size,
            } => global.render_bundle_encoder_set_vertex_buffer_with_id(
                render_bundle_encoder_id,
                slot,
                buffer,
                offset,
                size,
            ),
            RenderCommand::Draw {
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            } => global.render_bundle_encoder_draw_with_id(
                render_bundle_encoder_id,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            ),
            RenderCommand::DrawIndexed {
                index_count,
                instance_count,
                first_index,
                base_vertex,
                first_instance,
            } => global.render_bundle_encoder_draw_indexed_with_id(
                render_bundle_encoder_id,
                index_count,
                instance_count,
                first_index,
                base_vertex,
                first_instance,
            ),
            RenderCommand::DrawIndirect {
                indirect_buffer,
                indirect_offset,
            } => global.render_bundle_encoder_draw_indirect_with_id(
                render_bundle_encoder_id,
                indirect_buffer,
                indirect_offset,
            ),
            RenderCommand::DrawIndexedIndirect {
                indirect_buffer,
                indirect_offset,
            } => global.render_bundle_encoder_draw_indexed_indirect_with_id(
                render_bundle_encoder_id,
                indirect_buffer,
                indirect_offset,
            ),
        },
        RenderBundleEncoderCommand::DebugCommand(debug_command) => match debug_command {
            DebugCommand::PushDebugGroup(label) => global
                .render_bundle_encoder_push_debug_group_with_id(render_bundle_encoder_id, &label),
            DebugCommand::PopDebugGroup => {
                global.render_bundle_encoder_pop_debug_group_with_id(render_bundle_encoder_id)
            },
            DebugCommand::InsertDebugMarker(label) => global
                .render_bundle_encoder_insert_debug_marker_with_id(
                    render_bundle_encoder_id,
                    &label,
                ),
        },
    }
}

pub(crate) fn handle_compute_pass_command(
    global: &Global,
    compute_pass_id: ComputePassEncoderId,
    command: ComputePassEncoderCommand,
) -> Result<(), PassStateError> {
    match command {
        ComputePassEncoderCommand::BindingCommand(binding_command) => match binding_command {
            BindingCommand::SetBindGroup {
                index,
                bind_group,
                dynamic_offsets,
            } => global.compute_pass_set_bind_group_with_id(
                compute_pass_id,
                index,
                bind_group,
                &dynamic_offsets,
            ),
            BindingCommand::SetImmediates { range_offset, data } => {
                global.compute_pass_set_immediates_with_id(compute_pass_id, range_offset, &data)
            },
        },
        ComputePassEncoderCommand::SetPipeline(pipeline) => {
            global.compute_pass_set_pipeline_with_id(compute_pass_id, pipeline)
        },
        ComputePassEncoderCommand::DispatchWorkgroups {
            workgroup_count_x,
            workgroup_count_y,
            workgroup_count_z,
        } => global.compute_pass_dispatch_workgroups_with_id(
            compute_pass_id,
            workgroup_count_x,
            workgroup_count_y,
            workgroup_count_z,
        ),
        ComputePassEncoderCommand::DispatchWorkgroupsIndirect {
            indirect_buffer,
            indirect_offset,
        } => global.compute_pass_dispatch_workgroups_indirect_with_id(
            compute_pass_id,
            indirect_buffer,
            indirect_offset,
        ),
        ComputePassEncoderCommand::DebugCommand(debug_command) => match debug_command {
            DebugCommand::PushDebugGroup(label) => {
                global.compute_pass_push_debug_group_with_id(compute_pass_id, &label, 0)
            },
            DebugCommand::PopDebugGroup => {
                global.compute_pass_pop_debug_group_with_id(compute_pass_id)
            },
            DebugCommand::InsertDebugMarker(label) => {
                global.compute_pass_insert_debug_marker_with_id(compute_pass_id, &label, 0)
            },
        },
    }
}

pub(crate) fn handle_command_encoder_command(
    global: &Global,
    command_encoder_id: CommandEncoderId,
    command: CommandEncoderCommand,
) -> Result<(), EncoderStateError> {
    match command {
        CommandEncoderCommand::CopyBufferToBuffer {
            source,
            source_offset,
            destination,
            destination_offset,
            size,
        } => global.command_encoder_copy_buffer_to_buffer(
            command_encoder_id,
            source,
            source_offset,
            destination,
            destination_offset,
            size,
        ),
        CommandEncoderCommand::CopyBufferToTexture {
            source,
            destination,
            copy_size,
        } => global.command_encoder_copy_buffer_to_texture(
            command_encoder_id,
            &source,
            &destination,
            &copy_size,
        ),
        CommandEncoderCommand::CopyTextureToBuffer {
            source,
            destination,
            copy_size,
        } => global.command_encoder_copy_texture_to_buffer(
            command_encoder_id,
            &source,
            &destination,
            &copy_size,
        ),
        CommandEncoderCommand::CopyTextureToTexture {
            source,
            destination,
            copy_size,
        } => global.command_encoder_copy_texture_to_texture(
            command_encoder_id,
            &source,
            &destination,
            &copy_size,
        ),
        CommandEncoderCommand::ClearBuffer {
            buffer,
            offset,
            size,
        } => global.command_encoder_clear_buffer(command_encoder_id, buffer, offset, size),
        CommandEncoderCommand::ResolveQuerySet {
            query_set,
            first_query,
            query_count,
            destination,
            destination_offset,
        } => global.command_encoder_resolve_query_set(
            command_encoder_id,
            query_set,
            first_query,
            query_count,
            destination,
            destination_offset,
        ),
        CommandEncoderCommand::DebugCommand(debug_command) => match debug_command {
            DebugCommand::PushDebugGroup(label) => {
                global.command_encoder_push_debug_group(command_encoder_id, &label)
            },
            DebugCommand::PopDebugGroup => {
                global.command_encoder_pop_debug_group(command_encoder_id)
            },
            DebugCommand::InsertDebugMarker(label) => {
                global.command_encoder_insert_debug_marker(command_encoder_id, &label)
            },
        },
    }
}
