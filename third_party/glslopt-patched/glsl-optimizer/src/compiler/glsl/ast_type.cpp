/*
 * Copyright © 2010 Intel Corporation
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice (including the next
 * paragraph) shall be included in all copies or substantial portions of the
 * Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */

#include "ast.h"

void
ast_type_specifier::print(void) const
{
   if (structure) {
      structure->print();
   } else {
      printf("%s ", type_name);
   }

   if (array_specifier) {
      array_specifier->print();
   }
}

bool
ast_fully_specified_type::has_qualifiers(_mesa_glsl_parse_state *state) const
{
   /* 'subroutine' isnt a real qualifier. */
   ast_type_qualifier subroutine_only;
   subroutine_only.flags.i = 0;
   subroutine_only.flags.q.subroutine = 1;
   if (state->has_explicit_uniform_location()) {
      subroutine_only.flags.q.explicit_index = 1;
   }
   return (this->qualifier.flags.i & ~subroutine_only.flags.i) != 0;
}

bool ast_type_qualifier::has_interpolation() const
{
   return this->flags.q.smooth
          || this->flags.q.flat
          || this->flags.q.noperspective;
}

bool
ast_type_qualifier::has_layout() const
{
   return this->flags.q.origin_upper_left
          || this->flags.q.pixel_center_integer
          || this->flags.q.depth_type
          || this->flags.q.std140
          || this->flags.q.std430
          || this->flags.q.shared
          || this->flags.q.column_major
          || this->flags.q.row_major
          || this->flags.q.packed
          || this->flags.q.bindless_sampler
          || this->flags.q.bindless_image
          || this->flags.q.bound_sampler
          || this->flags.q.bound_image
          || this->flags.q.explicit_align
          || this->flags.q.explicit_component
          || this->flags.q.explicit_location
          || this->flags.q.explicit_image_format
          || this->flags.q.explicit_index
          || this->flags.q.explicit_binding
          || this->flags.q.explicit_offset
          || this->flags.q.explicit_stream
          || this->flags.q.explicit_xfb_buffer
          || this->flags.q.explicit_xfb_offset
          || this->flags.q.explicit_xfb_stride;
}

bool
ast_type_qualifier::has_storage() const
{
   return this->flags.q.constant
          || this->flags.q.attribute
          || this->flags.q.varying
          || this->flags.q.in
          || this->flags.q.out
          || this->flags.q.uniform
          || this->flags.q.buffer
          || this->flags.q.shared_storage;
}

bool
ast_type_qualifier::has_auxiliary_storage() const
{
   return this->flags.q.centroid
          || this->flags.q.sample
          || this->flags.q.patch;
}

bool ast_type_qualifier::has_memory() const
{
   return this->flags.q.coherent
          || this->flags.q._volatile
          || this->flags.q.restrict_flag
          || this->flags.q.read_only
          || this->flags.q.write_only;
}

bool ast_type_qualifier::is_subroutine_decl() const
{
   return this->flags.q.subroutine && !this->subroutine_list;
}

static bool
validate_prim_type(YYLTYPE *loc,
                   _mesa_glsl_parse_state *state,
                   const ast_type_qualifier &qualifier,
                   const ast_type_qualifier &new_qualifier)
{
   /* Input layout qualifiers can be specified multiple
    * times in separate declarations, as long as they match.
    */
   if (qualifier.flags.q.prim_type && new_qualifier.flags.q.prim_type
       && qualifier.prim_type != new_qualifier.prim_type) {
      _mesa_glsl_error(loc, state,
                       "conflicting input primitive %s specified",
                       state->stage == MESA_SHADER_GEOMETRY ?
                       "type" : "mode");
      return false;
   }

   return true;
}

static bool
validate_vertex_spacing(YYLTYPE *loc,
                        _mesa_glsl_parse_state *state,
                        const ast_type_qualifier &qualifier,
                        const ast_type_qualifier &new_qualifier)
{
   if (qualifier.flags.q.vertex_spacing && new_qualifier.flags.q.vertex_spacing
       && qualifier.vertex_spacing != new_qualifier.vertex_spacing) {
      _mesa_glsl_error(loc, state,
                       "conflicting vertex spacing specified");
      return false;
   }

   return true;
}

static bool
validate_ordering(YYLTYPE *loc,
                  _mesa_glsl_parse_state *state,
                  const ast_type_qualifier &qualifier,
                  const ast_type_qualifier &new_qualifier)
{
   if (qualifier.flags.q.ordering && new_qualifier.flags.q.ordering
       && qualifier.ordering != new_qualifier.ordering) {
      _mesa_glsl_error(loc, state,
                       "conflicting ordering specified");
      return false;
   }

   return true;
}

static bool
validate_point_mode(ASSERTED const ast_type_qualifier &qualifier,
                    ASSERTED const ast_type_qualifier &new_qualifier)
{
   /* Point mode can only be true if the flag is set. */
   assert (!qualifier.flags.q.point_mode || !new_qualifier.flags.q.point_mode
           || (qualifier.point_mode && new_qualifier.point_mode));

   return true;
}

static void
merge_bindless_qualifier(_mesa_glsl_parse_state *state)
{
   if (state->default_uniform_qualifier->flags.q.bindless_sampler) {
      state->bindless_sampler_specified = true;
      state->default_uniform_qualifier->flags.q.bindless_sampler = false;
   }

   if (state->default_uniform_qualifier->flags.q.bindless_image) {
      state->bindless_image_specified = true;
      state->default_uniform_qualifier->flags.q.bindless_image = false;
   }

   if (state->default_uniform_qualifier->flags.q.bound_sampler) {
      state->bound_sampler_specified = true;
      state->default_uniform_qualifier->flags.q.bound_sampler = false;
   }

   if (state->default_uniform_qualifier->flags.q.bound_image) {
      state->bound_image_specified = true;
      state->default_uniform_qualifier->flags.q.bound_image = false;
   }
}

/**
 * This function merges duplicate layout identifiers.
 *
 * It deals with duplicates within a single layout qualifier, among multiple
 * layout qualifiers on a single declaration and on several declarations for
 * the same variable.
 *
 * The is_single_layout_merge and is_multiple_layouts_merge parameters are
 * used to differentiate among them.
 */
bool
ast_type_qualifier::merge_qualifier(YYLTYPE *loc,
                                    _mesa_glsl_parse_state *state,
                                    const ast_type_qualifier &q,
                                    bool is_single_layout_merge,
                                    bool is_multiple_layouts_merge)
{
   bool r = true;
   ast_type_qualifier ubo_mat_mask;
   ubo_mat_mask.flags.i = 0;
   ubo_mat_mask.flags.q.row_major = 1;
   ubo_mat_mask.flags.q.column_major = 1;

   ast_type_qualifier ubo_layout_mask;
   ubo_layout_mask.flags.i = 0;
   ubo_layout_mask.flags.q.std140 = 1;
   ubo_layout_mask.flags.q.packed = 1;
   ubo_layout_mask.flags.q.shared = 1;
   ubo_layout_mask.flags.q.std430 = 1;

   ast_type_qualifier ubo_binding_mask;
   ubo_binding_mask.flags.i = 0;
   ubo_binding_mask.flags.q.explicit_binding = 1;
   ubo_binding_mask.flags.q.explicit_offset = 1;

   ast_type_qualifier stream_layout_mask;
   stream_layout_mask.flags.i = 0;
   stream_layout_mask.flags.q.stream = 1;

   /* FIXME: We should probably do interface and function param validation
    * separately.
    */
   ast_type_qualifier input_layout_mask;
   input_layout_mask.flags.i = 0;
   input_layout_mask.flags.q.centroid = 1;
   /* Function params can have constant */
   input_layout_mask.flags.q.constant = 1;
   input_layout_mask.flags.q.explicit_component = 1;
   input_layout_mask.flags.q.explicit_location = 1;
   input_layout_mask.flags.q.flat = 1;
   input_layout_mask.flags.q.in = 1;
   input_layout_mask.flags.q.invariant = 1;
   input_layout_mask.flags.q.noperspective = 1;
   input_layout_mask.flags.q.origin_upper_left = 1;
   /* Function params 'inout' will set this */
   input_layout_mask.flags.q.out = 1;
   input_layout_mask.flags.q.patch = 1;
   input_layout_mask.flags.q.pixel_center_integer = 1;
   input_layout_mask.flags.q.precise = 1;
   input_layout_mask.flags.q.sample = 1;
   input_layout_mask.flags.q.smooth = 1;
   input_layout_mask.flags.q.non_coherent = 1;

   if (state->has_bindless()) {
      /* Allow to use image qualifiers with shader inputs/outputs. */
      input_layout_mask.flags.q.coherent = 1;
      input_layout_mask.flags.q._volatile = 1;
      input_layout_mask.flags.q.restrict_flag = 1;
      input_layout_mask.flags.q.read_only = 1;
      input_layout_mask.flags.q.write_only = 1;
      input_layout_mask.flags.q.explicit_image_format = 1;
   }

   /* Uniform block layout qualifiers get to overwrite each
    * other (rightmost having priority), while all other
    * qualifiers currently don't allow duplicates.
    */
   ast_type_qualifier allowed_duplicates_mask;
   allowed_duplicates_mask.flags.i =
      ubo_mat_mask.flags.i |
      ubo_layout_mask.flags.i |
      ubo_binding_mask.flags.i;

   /* Geometry shaders can have several layout qualifiers
    * assigning different stream values.
    */
   if (state->stage == MESA_SHADER_GEOMETRY) {
      allowed_duplicates_mask.flags.i |=
         stream_layout_mask.flags.i;
   }

   if (is_single_layout_merge && !state->has_enhanced_layouts() &&
       (this->flags.i & q.flags.i & ~allowed_duplicates_mask.flags.i) != 0) {
      _mesa_glsl_error(loc, state, "duplicate layout qualifiers used");
      return false;
   }

   if (is_multiple_layouts_merge && !state->has_420pack_or_es31()) {
      _mesa_glsl_error(loc, state,
                       "duplicate layout(...) qualifiers");
      return false;
   }

   if (q.flags.q.prim_type) {
      r &= validate_prim_type(loc, state, *this, q);
      this->flags.q.prim_type = 1;
      this->prim_type = q.prim_type;
   }

   if (q.flags.q.max_vertices) {
      if (this->flags.q.max_vertices
          && !is_single_layout_merge && !is_multiple_layouts_merge) {
         this->max_vertices->merge_qualifier(q.max_vertices);
      } else {
         this->flags.q.max_vertices = 1;
         this->max_vertices = q.max_vertices;
      }
   }

   if (q.subroutine_list) {
      if (this->subroutine_list) {
         _mesa_glsl_error(loc, state,
                          "conflicting subroutine qualifiers used");
      } else {
         this->subroutine_list = q.subroutine_list;
      }
   }

   if (q.flags.q.invocations) {
      if (this->flags.q.invocations
          && !is_single_layout_merge && !is_multiple_layouts_merge) {
         this->invocations->merge_qualifier(q.invocations);
      } else {
         this->flags.q.invocations = 1;
         this->invocations = q.invocations;
      }
   }

   if (state->stage == MESA_SHADER_GEOMETRY &&
       state->has_explicit_attrib_stream()) {
      if (!this->flags.q.explicit_stream) {
         if (q.flags.q.stream) {
            this->flags.q.stream = 1;
            this->stream = q.stream;
         } else if (!this->flags.q.stream && this->flags.q.out &&
                    !this->flags.q.in) {
            /* Assign default global stream value */
            this->flags.q.stream = 1;
            this->stream = state->out_qualifier->stream;
         }
      }
   }

   if (state->has_enhanced_layouts()) {
      if (!this->flags.q.explicit_xfb_buffer) {
         if (q.flags.q.xfb_buffer) {
            this->flags.q.xfb_buffer = 1;
            this->xfb_buffer = q.xfb_buffer;
         } else if (!this->flags.q.xfb_buffer && this->flags.q.out &&
                    !this->flags.q.in) {
            /* Assign global xfb_buffer value */
            this->flags.q.xfb_buffer = 1;
            this->xfb_buffer = state->out_qualifier->xfb_buffer;
         }
      }

      if (q.flags.q.explicit_xfb_stride) {
         this->flags.q.xfb_stride = 1;
         this->flags.q.explicit_xfb_stride = 1;
         this->xfb_stride = q.xfb_stride;
      }
   }

   if (q.flags.q.vertices) {
      if (this->flags.q.vertices
          && !is_single_layout_merge && !is_multiple_layouts_merge) {
         this->vertices->merge_qualifier(q.vertices);
      } else {
         this->flags.q.vertices = 1;
         this->vertices = q.vertices;
      }
   }

   if (q.flags.q.vertex_spacing) {
      r &= validate_vertex_spacing(loc, state, *this, q);
      this->flags.q.vertex_spacing = 1;
      this->vertex_spacing = q.vertex_spacing;
   }

   if (q.flags.q.ordering) {
      r &= validate_ordering(loc, state, *this, q);
      this->flags.q.ordering = 1;
      this->ordering = q.ordering;
   }

   if (q.flags.q.point_mode) {
      r &= validate_point_mode(*this, q);
      this->flags.q.point_mode = 1;
      this->point_mode = q.point_mode;
   }

   if (q.flags.q.early_fragment_tests)
      this->flags.q.early_fragment_tests = true;

   if ((q.flags.i & ubo_mat_mask.flags.i) != 0)
      this->flags.i &= ~ubo_mat_mask.flags.i;
   if ((q.flags.i & ubo_layout_mask.flags.i) != 0)
      this->flags.i &= ~ubo_layout_mask.flags.i;

   for (int i = 0; i < 3; i++) {
      if (q.flags.q.local_size & (1 << i)) {
         if (this->local_size[i]
             && !is_single_layout_merge && !is_multiple_layouts_merge) {
            this->local_size[i]->merge_qualifier(q.local_size[i]);
         } else {
            this->local_size[i] = q.local_size[i];
         }
      }
   }

   if (q.flags.q.local_size_variable)
      this->flags.q.local_size_variable = true;

   if (q.flags.q.bindless_sampler)
      this->flags.q.bindless_sampler = true;

   if (q.flags.q.bindless_image)
      this->flags.q.bindless_image = true;

   if (q.flags.q.bound_sampler)
      this->flags.q.bound_sampler = true;

   if (q.flags.q.bound_image)
      this->flags.q.bound_image = true;

   if (q.flags.q.derivative_group) {
      this->flags.q.derivative_group = true;
      this->derivative_group = q.derivative_group;
   }

   this->flags.i |= q.flags.i;

   if (this->flags.q.in &&
       (this->flags.i & ~input_layout_mask.flags.i) != 0) {
      _mesa_glsl_error(loc, state, "invalid input layout qualifier used");
      return false;
   }

   if (q.flags.q.explicit_align)
      this->align = q.align;

   if (q.flags.q.explicit_location)
      this->location = q.location;

   if (q.flags.q.explicit_index)
      this->index = q.index;

  if (q.flags.q.explicit_component)
      this->component = q.component;

   if (q.flags.q.explicit_binding)
      this->binding = q.binding;

   if (q.flags.q.explicit_offset || q.flags.q.explicit_xfb_offset)
      this->offset = q.offset;

   if (q.precision != ast_precision_none)
      this->precision = q.precision;

   if (q.flags.q.explicit_image_format) {
      this->image_format = q.image_format;
      this->image_base_type = q.image_base_type;
   }

   if (q.flags.q.bindless_sampler ||
       q.flags.q.bindless_image ||
       q.flags.q.bound_sampler ||
       q.flags.q.bound_image)
      merge_bindless_qualifier(state);

   if (state->EXT_gpu_shader4_enable &&
       state->stage == MESA_SHADER_FRAGMENT &&
       this->flags.q.varying && q.flags.q.out) {
      this->flags.q.varying = 0;
      this->flags.q.out = 1;
   }

   return r;
}

bool
ast_type_qualifier::validate_out_qualifier(YYLTYPE *loc,
                                           _mesa_glsl_parse_state *state)
{
   bool r = true;
   ast_type_qualifier valid_out_mask;
   valid_out_mask.flags.i = 0;

   switch (state->stage) {
   case MESA_SHADER_GEOMETRY:
      if (this->flags.q.prim_type) {
         /* Make sure this is a valid output primitive type. */
         switch (this->prim_type) {
         case GL_POINTS:
         case GL_LINE_STRIP:
         case GL_TRIANGLE_STRIP:
            break;
         default:
            r = false;
            _mesa_glsl_error(loc, state, "invalid geometry shader output "
                             "primitive type");
            break;
         }
      }

      valid_out_mask.flags.q.stream = 1;
      valid_out_mask.flags.q.explicit_stream = 1;
      valid_out_mask.flags.q.explicit_xfb_buffer = 1;
      valid_out_mask.flags.q.xfb_buffer = 1;
      valid_out_mask.flags.q.explicit_xfb_stride = 1;
      valid_out_mask.flags.q.xfb_stride = 1;
      valid_out_mask.flags.q.max_vertices = 1;
      valid_out_mask.flags.q.prim_type = 1;
      break;
   case MESA_SHADER_TESS_CTRL:
      valid_out_mask.flags.q.vertices = 1;
      valid_out_mask.flags.q.explicit_xfb_buffer = 1;
      valid_out_mask.flags.q.xfb_buffer = 1;
      valid_out_mask.flags.q.explicit_xfb_stride = 1;
      valid_out_mask.flags.q.xfb_stride = 1;
      break;
   case MESA_SHADER_TESS_EVAL:
   case MESA_SHADER_VERTEX:
      valid_out_mask.flags.q.explicit_xfb_buffer = 1;
      valid_out_mask.flags.q.xfb_buffer = 1;
      valid_out_mask.flags.q.explicit_xfb_stride = 1;
      valid_out_mask.flags.q.xfb_stride = 1;
      break;
   case MESA_SHADER_FRAGMENT:
      valid_out_mask.flags.q.blend_support = 1;
      break;
   default:
      r = false;
      _mesa_glsl_error(loc, state,
                       "out layout qualifiers only valid in "
                       "geometry, tessellation, vertex and fragment shaders");
   }

   /* Generate an error when invalid output layout qualifiers are used. */
   if ((this->flags.i & ~valid_out_mask.flags.i) != 0) {
      r = false;
      _mesa_glsl_error(loc, state, "invalid output layout qualifiers used");
   }

   return r;
}

bool
ast_type_qualifier::merge_into_out_qualifier(YYLTYPE *loc,
                                             _mesa_glsl_parse_state *state,
                                             ast_node* &node)
{
   const bool r = state->out_qualifier->merge_qualifier(loc, state,
                                                        *this, false);

   switch (state->stage) {
   case MESA_SHADER_GEOMETRY:
      /* Allow future assignments of global out's stream id value */
      state->out_qualifier->flags.q.explicit_stream = 0;
      break;
   case MESA_SHADER_TESS_CTRL:
      node = new(state->linalloc) ast_tcs_output_layout(*loc);
      break;
   default:
      break;
   }

   /* Allow future assignments of global out's */
   state->out_qualifier->flags.q.explicit_xfb_buffer = 0;
   state->out_qualifier->flags.q.explicit_xfb_stride = 0;

   return r;
}

bool
ast_type_qualifier::validate_in_qualifier(YYLTYPE *loc,
                                          _mesa_glsl_parse_state *state)
{
   bool r = true;
   ast_type_qualifier valid_in_mask;
   valid_in_mask.flags.i = 0;

   switch (state->stage) {
   case MESA_SHADER_TESS_EVAL:
      if (this->flags.q.prim_type) {
         /* Make sure this is a valid input primitive type. */
         switch (this->prim_type) {
         case GL_TRIANGLES:
         case GL_QUADS:
         case GL_ISOLINES:
            break;
         default:
            r = false;
            _mesa_glsl_error(loc, state,
                             "invalid tessellation evaluation "
                             "shader input primitive type");
            break;
         }
      }

      valid_in_mask.flags.q.prim_type = 1;
      valid_in_mask.flags.q.vertex_spacing = 1;
      valid_in_mask.flags.q.ordering = 1;
      valid_in_mask.flags.q.point_mode = 1;
      break;
   case MESA_SHADER_GEOMETRY:
      if (this->flags.q.prim_type) {
         /* Make sure this is a valid input primitive type. */
         switch (this->prim_type) {
         case GL_POINTS:
         case GL_LINES:
         case GL_LINES_ADJACENCY:
         case GL_TRIANGLES:
         case GL_TRIANGLES_ADJACENCY:
            break;
         default:
            r = false;
            _mesa_glsl_error(loc, state,
                             "invalid geometry shader input primitive type");
            break;
         }
      }

      valid_in_mask.flags.q.prim_type = 1;
      valid_in_mask.flags.q.invocations = 1;
      break;
   case MESA_SHADER_FRAGMENT:
      valid_in_mask.flags.q.early_fragment_tests = 1;
      valid_in_mask.flags.q.inner_coverage = 1;
      valid_in_mask.flags.q.post_depth_coverage = 1;
      valid_in_mask.flags.q.pixel_interlock_ordered = 1;
      valid_in_mask.flags.q.pixel_interlock_unordered = 1;
      valid_in_mask.flags.q.sample_interlock_ordered = 1;
      valid_in_mask.flags.q.sample_interlock_unordered = 1;
      break;
   case MESA_SHADER_COMPUTE:
      valid_in_mask.flags.q.local_size = 7;
      valid_in_mask.flags.q.local_size_variable = 1;
      valid_in_mask.flags.q.derivative_group = 1;
      break;
   default:
      r = false;
      _mesa_glsl_error(loc, state,
                       "input layout qualifiers only valid in "
                       "geometry, tessellation, fragment and compute shaders");
      break;
   }

   /* Generate an error when invalid input layout qualifiers are used. */
   if ((this->flags.i & ~valid_in_mask.flags.i) != 0) {
      r = false;
      _mesa_glsl_error(loc, state, "invalid input layout qualifiers used");
   }

   /* The checks below are also performed when merging but we want to spit an
    * error against the default global input qualifier as soon as we can, with
    * the closest error location in the shader.
    */
   r &= validate_prim_type(loc, state, *state->in_qualifier, *this);
   r &= validate_vertex_spacing(loc, state, *state->in_qualifier, *this);
   r &= validate_ordering(loc, state, *state->in_qualifier, *this);
   r &= validate_point_mode(*state->in_qualifier, *this);

   return r;
}

bool
ast_type_qualifier::merge_into_in_qualifier(YYLTYPE *loc,
                                            _mesa_glsl_parse_state *state,
                                            ast_node* &node)
{
   bool r = true;
   void *lin_ctx = state->linalloc;

   /* We create the gs_input_layout node before merging so, in the future, no
    * more repeated nodes will be created as we will have the flag set.
    */
   if (state->stage == MESA_SHADER_GEOMETRY
       && this->flags.q.prim_type && !state->in_qualifier->flags.q.prim_type) {
      node = new(lin_ctx) ast_gs_input_layout(*loc, this->prim_type);
   }

   r = state->in_qualifier->merge_qualifier(loc, state, *this, false);

   if (state->in_qualifier->flags.q.early_fragment_tests) {
      state->fs_early_fragment_tests = true;
      state->in_qualifier->flags.q.early_fragment_tests = false;
   }

   if (state->in_qualifier->flags.q.inner_coverage) {
      state->fs_inner_coverage = true;
      state->in_qualifier->flags.q.inner_coverage = false;
   }

   if (state->in_qualifier->flags.q.post_depth_coverage) {
      state->fs_post_depth_coverage = true;
      state->in_qualifier->flags.q.post_depth_coverage = false;
   }

   if (state->fs_inner_coverage && state->fs_post_depth_coverage) {
      _mesa_glsl_error(loc, state,
                       "inner_coverage & post_depth_coverage layout qualifiers "
                       "are mutally exclusives");
      r = false;
   }

   if (state->in_qualifier->flags.q.pixel_interlock_ordered) {
      state->fs_pixel_interlock_ordered = true;
      state->in_qualifier->flags.q.pixel_interlock_ordered = false;
   }

   if (state->in_qualifier->flags.q.pixel_interlock_unordered) {
      state->fs_pixel_interlock_unordered = true;
      state->in_qualifier->flags.q.pixel_interlock_unordered = false;
   }

   if (state->in_qualifier->flags.q.sample_interlock_ordered) {
      state->fs_sample_interlock_ordered = true;
      state->in_qualifier->flags.q.sample_interlock_ordered = false;
   }

   if (state->in_qualifier->flags.q.sample_interlock_unordered) {
      state->fs_sample_interlock_unordered = true;
      state->in_qualifier->flags.q.sample_interlock_unordered = false;
   }

   if (state->fs_pixel_interlock_ordered +
       state->fs_pixel_interlock_unordered +
       state->fs_sample_interlock_ordered +
       state->fs_sample_interlock_unordered > 1) {
      _mesa_glsl_error(loc, state,
                       "only one interlock mode can be used at any time.");
      r = false;
   }

   if (state->in_qualifier->flags.q.derivative_group) {
      if (state->cs_derivative_group != DERIVATIVE_GROUP_NONE) {
         if (state->in_qualifier->derivative_group != DERIVATIVE_GROUP_NONE &&
             state->cs_derivative_group != state->in_qualifier->derivative_group) {
            _mesa_glsl_error(loc, state,
                             "conflicting derivative groups.");
            r = false;
         }
      } else {
         state->cs_derivative_group = state->in_qualifier->derivative_group;
      }
   }

   /* We allow the creation of multiple cs_input_layout nodes. Coherence among
    * all existing nodes is checked later, when the AST node is transformed
    * into HIR.
    */
   if (state->in_qualifier->flags.q.local_size) {
      node = new(lin_ctx) ast_cs_input_layout(*loc,
                                              state->in_qualifier->local_size);
      state->in_qualifier->flags.q.local_size = 0;
      for (int i = 0; i < 3; i++)
         state->in_qualifier->local_size[i] = NULL;
   }

   if (state->in_qualifier->flags.q.local_size_variable) {
      state->cs_input_local_size_variable_specified = true;
      state->in_qualifier->flags.q.local_size_variable = false;
   }

   return r;
}

bool
ast_type_qualifier::push_to_global(YYLTYPE *loc,
                                   _mesa_glsl_parse_state *state)
{
   if (this->flags.q.xfb_stride) {
      this->flags.q.xfb_stride = 0;

      unsigned buff_idx;
      if (process_qualifier_constant(state, loc, "xfb_buffer",
                                     this->xfb_buffer, &buff_idx)) {
         if (state->out_qualifier->out_xfb_stride[buff_idx]) {
            state->out_qualifier->out_xfb_stride[buff_idx]->merge_qualifier(
               new(state->linalloc) ast_layout_expression(*loc,
                                                          this->xfb_stride));
         } else {
            state->out_qualifier->out_xfb_stride[buff_idx] =
               new(state->linalloc) ast_layout_expression(*loc,
                                                          this->xfb_stride);
         }
      }
   }

   return true;
}

/**
 * Check if the current type qualifier has any illegal flags.
 *
 * If so, print an error message, followed by a list of illegal flags.
 *
 * \param message        The error message to print.
 * \param allowed_flags  A list of valid flags.
 */
bool
ast_type_qualifier::validate_flags(YYLTYPE *loc,
                                   _mesa_glsl_parse_state *state,
                                   const ast_type_qualifier &allowed_flags,
                                   const char *message, const char *name)
{
   ast_type_qualifier bad;
   bad.flags.i = this->flags.i & ~allowed_flags.flags.i;
   if (bad.flags.i == 0)
      return true;

   _mesa_glsl_error(loc, state,
                    "%s '%s':"
                    "%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s"
                    "%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s"
                    "%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s\n",
                    message, name,
                    bad.flags.q.invariant ? " invariant" : "",
                    bad.flags.q.precise ? " precise" : "",
                    bad.flags.q.constant ? " constant" : "",
                    bad.flags.q.attribute ? " attribute" : "",
                    bad.flags.q.varying ? " varying" : "",
                    bad.flags.q.in ? " in" : "",
                    bad.flags.q.out ? " out" : "",
                    bad.flags.q.centroid ? " centroid" : "",
                    bad.flags.q.sample ? " sample" : "",
                    bad.flags.q.patch ? " patch" : "",
                    bad.flags.q.uniform ? " uniform" : "",
                    bad.flags.q.buffer ? " buffer" : "",
                    bad.flags.q.shared_storage ? " shared_storage" : "",
                    bad.flags.q.smooth ? " smooth" : "",
                    bad.flags.q.flat ? " flat" : "",
                    bad.flags.q.noperspective ? " noperspective" : "",
                    bad.flags.q.origin_upper_left ? " origin_upper_left" : "",
                    bad.flags.q.pixel_center_integer ? " pixel_center_integer" : "",
                    bad.flags.q.explicit_align ? " align" : "",
                    bad.flags.q.explicit_component ? " component" : "",
                    bad.flags.q.explicit_location ? " location" : "",
                    bad.flags.q.explicit_index ? " index" : "",
                    bad.flags.q.explicit_binding ? " binding" : "",
                    bad.flags.q.explicit_offset ? " offset" : "",
                    bad.flags.q.depth_type ? " depth_type" : "",
                    bad.flags.q.std140 ? " std140" : "",
                    bad.flags.q.std430 ? " std430" : "",
                    bad.flags.q.shared ? " shared" : "",
                    bad.flags.q.packed ? " packed" : "",
                    bad.flags.q.column_major ? " column_major" : "",
                    bad.flags.q.row_major ? " row_major" : "",
                    bad.flags.q.prim_type ? " prim_type" : "",
                    bad.flags.q.max_vertices ? " max_vertices" : "",
                    bad.flags.q.local_size ? " local_size" : "",
                    bad.flags.q.local_size_variable ? " local_size_variable" : "",
                    bad.flags.q.early_fragment_tests ? " early_fragment_tests" : "",
                    bad.flags.q.explicit_image_format ? " image_format" : "",
                    bad.flags.q.coherent ? " coherent" : "",
                    bad.flags.q._volatile ? " _volatile" : "",
                    bad.flags.q.restrict_flag ? " restrict_flag" : "",
                    bad.flags.q.read_only ? " read_only" : "",
                    bad.flags.q.write_only ? " write_only" : "",
                    bad.flags.q.invocations ? " invocations" : "",
                    bad.flags.q.stream ? " stream" : "",
                    bad.flags.q.explicit_stream ? " stream" : "",
                    bad.flags.q.explicit_xfb_offset ? " xfb_offset" : "",
                    bad.flags.q.xfb_buffer ? " xfb_buffer" : "",
                    bad.flags.q.explicit_xfb_buffer ? " xfb_buffer" : "",
                    bad.flags.q.xfb_stride ? " xfb_stride" : "",
                    bad.flags.q.explicit_xfb_stride ? " xfb_stride" : "",
                    bad.flags.q.vertex_spacing ? " vertex_spacing" : "",
                    bad.flags.q.ordering ? " ordering" : "",
                    bad.flags.q.point_mode ? " point_mode" : "",
                    bad.flags.q.vertices ? " vertices" : "",
                    bad.flags.q.subroutine ? " subroutine" : "",
                    bad.flags.q.blend_support ? " blend_support" : "",
                    bad.flags.q.inner_coverage ? " inner_coverage" : "",
                    bad.flags.q.bindless_sampler ? " bindless_sampler" : "",
                    bad.flags.q.bindless_image ? " bindless_image" : "",
                    bad.flags.q.bound_sampler ? " bound_sampler" : "",
                    bad.flags.q.bound_image ? " bound_image" : "",
                    bad.flags.q.post_depth_coverage ? " post_depth_coverage" : "",
                    bad.flags.q.pixel_interlock_ordered ? " pixel_interlock_ordered" : "",
                    bad.flags.q.pixel_interlock_unordered ? " pixel_interlock_unordered": "",
                    bad.flags.q.sample_interlock_ordered ? " sample_interlock_ordered": "",
                    bad.flags.q.sample_interlock_unordered ? " sample_interlock_unordered": "",
                    bad.flags.q.non_coherent ? " noncoherent" : "");
   return false;
}

bool
ast_layout_expression::process_qualifier_constant(struct _mesa_glsl_parse_state *state,
                                                  const char *qual_indentifier,
                                                  unsigned *value,
                                                  bool can_be_zero)
{
   int min_value = 0;
   bool first_pass = true;
   *value = 0;

   if (!can_be_zero)
      min_value = 1;

   for (exec_node *node = layout_const_expressions.get_head_raw();
        !node->is_tail_sentinel(); node = node->next) {

      exec_list dummy_instructions;
      ast_node *const_expression = exec_node_data(ast_node, node, link);

      ir_rvalue *const ir = const_expression->hir(&dummy_instructions, state);

      ir_constant *const const_int =
         ir->constant_expression_value(ralloc_parent(ir));

      if (const_int == NULL || !const_int->type->is_integer_32()) {
         YYLTYPE loc = const_expression->get_location();
         _mesa_glsl_error(&loc, state, "%s must be an integral constant "
                          "expression", qual_indentifier);
         return false;
      }

      if (const_int->value.i[0] < min_value) {
         YYLTYPE loc = const_expression->get_location();
         _mesa_glsl_error(&loc, state, "%s layout qualifier is invalid "
                          "(%d < %d)", qual_indentifier,
                          const_int->value.i[0], min_value);
         return false;
      }

      if (!first_pass && *value != const_int->value.u[0]) {
         YYLTYPE loc = const_expression->get_location();
         _mesa_glsl_error(&loc, state, "%s layout qualifier does not "
		          "match previous declaration (%d vs %d)",
                          qual_indentifier, *value, const_int->value.i[0]);
         return false;
      } else {
         first_pass = false;
         *value = const_int->value.u[0];
      }

      /* If the location is const (and we've verified that
       * it is) then no instructions should have been emitted
       * when we converted it to HIR. If they were emitted,
       * then either the location isn't const after all, or
       * we are emitting unnecessary instructions.
       */
      assert(dummy_instructions.is_empty());
   }

   return true;
}

bool
process_qualifier_constant(struct _mesa_glsl_parse_state *state,
                           YYLTYPE *loc,
                           const char *qual_indentifier,
                           ast_expression *const_expression,
                           unsigned *value)
{
   exec_list dummy_instructions;

   if (const_expression == NULL) {
      *value = 0;
      return true;
   }

   ir_rvalue *const ir = const_expression->hir(&dummy_instructions, state);

   ir_constant *const const_int =
      ir->constant_expression_value(ralloc_parent(ir));
   if (const_int == NULL || !const_int->type->is_integer_32()) {
      _mesa_glsl_error(loc, state, "%s must be an integral constant "
                       "expression", qual_indentifier);
      return false;
   }

   if (const_int->value.i[0] < 0) {
      _mesa_glsl_error(loc, state, "%s layout qualifier is invalid (%d < 0)",
                       qual_indentifier, const_int->value.u[0]);
      return false;
   }

   /* If the location is const (and we've verified that
    * it is) then no instructions should have been emitted
    * when we converted it to HIR. If they were emitted,
    * then either the location isn't const after all, or
    * we are emitting unnecessary instructions.
    */
   assert(dummy_instructions.is_empty());

   *value = const_int->value.u[0];
   return true;
}
