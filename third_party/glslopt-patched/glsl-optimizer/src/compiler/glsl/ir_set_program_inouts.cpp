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

/**
 * \file ir_set_program_inouts.cpp
 *
 * Sets the inputs_read and outputs_written of Mesa programs.
 *
 * Mesa programs (gl_program, not gl_shader_program) have a set of
 * flags indicating which varyings are read and written.  Computing
 * which are actually read from some sort of backend code can be
 * tricky when variable array indexing involved.  So this pass
 * provides support for setting inputs_read and outputs_written right
 * from the GLSL IR.
 */

#include "ir.h"
#include "ir_visitor.h"
#include "compiler/glsl_types.h"
#include "main/mtypes.h"

namespace {

class ir_set_program_inouts_visitor : public ir_hierarchical_visitor {
public:
   ir_set_program_inouts_visitor(struct gl_program *prog,
                                 gl_shader_stage shader_stage)
   {
      this->prog = prog;
      this->shader_stage = shader_stage;
   }
   ~ir_set_program_inouts_visitor()
   {
   }

   virtual ir_visitor_status visit_enter(ir_dereference_array *);
   virtual ir_visitor_status visit_enter(ir_function_signature *);
   virtual ir_visitor_status visit_enter(ir_discard *);
   virtual ir_visitor_status visit_enter(ir_texture *);
   virtual ir_visitor_status visit(ir_dereference_variable *);

private:
   void mark_whole_variable(ir_variable *var);
   bool try_mark_partial_variable(ir_variable *var, ir_rvalue *index);

   struct gl_program *prog;
   gl_shader_stage shader_stage;
};

} /* anonymous namespace */

static inline bool
is_shader_inout(ir_variable *var)
{
   return var->data.mode == ir_var_shader_in ||
          var->data.mode == ir_var_shader_out ||
          var->data.mode == ir_var_system_value;
}

static void
mark(struct gl_program *prog, ir_variable *var, int offset, int len,
     gl_shader_stage stage)
{
   /* As of GLSL 1.20, varyings can only be floats, floating-point
    * vectors or matrices, or arrays of them.  For Mesa programs using
    * inputs_read/outputs_written, everything but matrices uses one
    * slot, while matrices use a slot per column.  Presumably
    * something doing a more clever packing would use something other
    * than inputs_read/outputs_written.
    */

   for (int i = 0; i < len; i++) {
      assert(var->data.location != -1);

      int idx = var->data.location + offset + i;
      bool is_patch_generic = var->data.patch &&
                              idx != VARYING_SLOT_TESS_LEVEL_INNER &&
                              idx != VARYING_SLOT_TESS_LEVEL_OUTER &&
                              idx != VARYING_SLOT_BOUNDING_BOX0 &&
                              idx != VARYING_SLOT_BOUNDING_BOX1;
      GLbitfield64 bitfield;

      if (is_patch_generic) {
         assert(idx >= VARYING_SLOT_PATCH0 && idx < VARYING_SLOT_TESS_MAX);
         bitfield = BITFIELD64_BIT(idx - VARYING_SLOT_PATCH0);
      }
      else {
         assert(idx < VARYING_SLOT_MAX);
         bitfield = BITFIELD64_BIT(idx);
      }

      if (var->data.mode == ir_var_shader_in) {
         if (is_patch_generic)
            prog->info.patch_inputs_read |= bitfield;
         else
            prog->info.inputs_read |= bitfield;

         /* double inputs read is only for vertex inputs */
         if (stage == MESA_SHADER_VERTEX &&
             var->type->without_array()->is_dual_slot())
            prog->DualSlotInputs |= bitfield;

         if (stage == MESA_SHADER_FRAGMENT) {
            prog->info.fs.uses_sample_qualifier |= var->data.sample;
         }
      } else if (var->data.mode == ir_var_system_value) {
         prog->info.system_values_read |= bitfield;
      } else {
         assert(var->data.mode == ir_var_shader_out);
         if (is_patch_generic) {
            prog->info.patch_outputs_written |= bitfield;
         } else if (!var->data.read_only) {
            prog->info.outputs_written |= bitfield;
            if (var->data.index > 0)
               prog->SecondaryOutputsWritten |= bitfield;
         }

         if (var->data.fb_fetch_output)
            prog->info.outputs_read |= bitfield;
      }
   }
}

/**
 * Mark an entire variable as used.  Caller must ensure that the variable
 * represents a shader input or output.
 */
void
ir_set_program_inouts_visitor::mark_whole_variable(ir_variable *var)
{
   const glsl_type *type = var->type;
   bool is_vertex_input = false;
   if (this->shader_stage == MESA_SHADER_GEOMETRY &&
       var->data.mode == ir_var_shader_in && type->is_array()) {
      type = type->fields.array;
   }

   if (this->shader_stage == MESA_SHADER_TESS_CTRL &&
       var->data.mode == ir_var_shader_in) {
      assert(type->is_array());
      type = type->fields.array;
   }

   if (this->shader_stage == MESA_SHADER_TESS_CTRL &&
       var->data.mode == ir_var_shader_out && !var->data.patch) {
      assert(type->is_array());
      type = type->fields.array;
   }

   if (this->shader_stage == MESA_SHADER_TESS_EVAL &&
       var->data.mode == ir_var_shader_in && !var->data.patch) {
      assert(type->is_array());
      type = type->fields.array;
   }

   if (this->shader_stage == MESA_SHADER_VERTEX &&
       var->data.mode == ir_var_shader_in)
      is_vertex_input = true;

   mark(this->prog, var, 0, type->count_attribute_slots(is_vertex_input),
        this->shader_stage);
}

/* Default handler: Mark all the locations in the variable as used. */
ir_visitor_status
ir_set_program_inouts_visitor::visit(ir_dereference_variable *ir)
{
   if (!is_shader_inout(ir->var))
      return visit_continue;

   mark_whole_variable(ir->var);

   return visit_continue;
}

/**
 * Try to mark a portion of the given variable as used.  Caller must ensure
 * that the variable represents a shader input or output which can be indexed
 * into in array fashion (an array or matrix).  For the purpose of geometry
 * shader inputs (which are always arrays*), this means that the array element
 * must be something that can be indexed into in array fashion.
 *
 * *Except gl_PrimitiveIDIn, as noted below.
 *
 * For tessellation control shaders all inputs and non-patch outputs are
 * arrays. For tessellation evaluation shaders non-patch inputs are arrays.
 *
 * If the index can't be interpreted as a constant, or some other problem
 * occurs, then nothing will be marked and false will be returned.
 */
bool
ir_set_program_inouts_visitor::try_mark_partial_variable(ir_variable *var,
                                                         ir_rvalue *index)
{
   const glsl_type *type = var->type;

   if (this->shader_stage == MESA_SHADER_GEOMETRY &&
       var->data.mode == ir_var_shader_in) {
      /* The only geometry shader input that is not an array is
       * gl_PrimitiveIDIn, and in that case, this code will never be reached,
       * because gl_PrimitiveIDIn can't be indexed into in array fashion.
       */
      assert(type->is_array());
      type = type->fields.array;
   }

   if (this->shader_stage == MESA_SHADER_TESS_CTRL &&
       var->data.mode == ir_var_shader_in) {
      assert(type->is_array());
      type = type->fields.array;
   }

   if (this->shader_stage == MESA_SHADER_TESS_CTRL &&
       var->data.mode == ir_var_shader_out && !var->data.patch) {
      assert(type->is_array());
      type = type->fields.array;
   }

   if (this->shader_stage == MESA_SHADER_TESS_EVAL &&
       var->data.mode == ir_var_shader_in && !var->data.patch) {
      assert(type->is_array());
      type = type->fields.array;
   }

   /* TODO: implement proper arrays of arrays support
    * for now let the caller mark whole variable as used.
    */
   if (type->is_array() && type->fields.array->is_array())
      return false;

   /* The code below only handles:
    *
    * - Indexing into matrices
    * - Indexing into arrays of (matrices, vectors, or scalars)
    *
    * All other possibilities are either prohibited by GLSL (vertex inputs and
    * fragment outputs can't be structs) or should have been eliminated by
    * lowering passes (do_vec_index_to_swizzle() gets rid of indexing into
    * vectors, and lower_packed_varyings() gets rid of structs that occur in
    * varyings).
    *
    * However, we don't use varying packing in all cases - tessellation
    * shaders bypass it.  This means we'll see varying structs and arrays
    * of structs here.  For now, we just give up so the caller marks the
    * entire variable as used.
    */
   if (!(type->is_matrix() ||
        (type->is_array() &&
         (type->fields.array->is_numeric() ||
          type->fields.array->is_boolean())))) {

      /* If we don't know how to handle this case, give up and let the
       * caller mark the whole variable as used.
       */
      return false;
   }

   ir_constant *index_as_constant = index->as_constant();
   if (!index_as_constant)
      return false;

   unsigned elem_width;
   unsigned num_elems;
   if (type->is_array()) {
      num_elems = type->length;
      if (type->fields.array->is_matrix())
         elem_width = type->fields.array->matrix_columns;
      else
         elem_width = 1;
   } else {
      num_elems = type->matrix_columns;
      elem_width = 1;
   }

   if (index_as_constant->value.u[0] >= num_elems) {
      /* Constant index outside the bounds of the matrix/array.  This could
       * arise as a result of constant folding of a legal GLSL program.
       *
       * Even though the spec says that indexing outside the bounds of a
       * matrix/array results in undefined behaviour, we don't want to pass
       * out-of-range values to mark() (since this could result in slots that
       * don't exist being marked as used), so just let the caller mark the
       * whole variable as used.
       */
      return false;
   }

   /* double element width for double types that takes two slots */
   if (this->shader_stage != MESA_SHADER_VERTEX ||
       var->data.mode != ir_var_shader_in) {
      if (type->without_array()->is_dual_slot())
	 elem_width *= 2;
   }

   mark(this->prog, var, index_as_constant->value.u[0] * elem_width,
        elem_width, this->shader_stage);
   return true;
}

static bool
is_multiple_vertices(gl_shader_stage stage, ir_variable *var)
{
   if (var->data.patch)
      return false;

   if (var->data.mode == ir_var_shader_in)
      return stage == MESA_SHADER_GEOMETRY ||
             stage == MESA_SHADER_TESS_CTRL ||
             stage == MESA_SHADER_TESS_EVAL;
   if (var->data.mode == ir_var_shader_out)
      return stage == MESA_SHADER_TESS_CTRL;

   return false;
}

ir_visitor_status
ir_set_program_inouts_visitor::visit_enter(ir_dereference_array *ir)
{
   /* Note: for geometry shader inputs, lower_named_interface_blocks may
    * create 2D arrays, so we need to be able to handle those.  2D arrays
    * shouldn't be able to crop up for any other reason.
    */
   if (ir_dereference_array * const inner_array =
       ir->array->as_dereference_array()) {
      /*          ir => foo[i][j]
       * inner_array => foo[i]
       */
      if (ir_dereference_variable * const deref_var =
          inner_array->array->as_dereference_variable()) {
         if (is_multiple_vertices(this->shader_stage, deref_var->var)) {
            /* foo is a geometry or tessellation shader input, so i is
             * the vertex, and j the part of the input we're accessing.
             */
            if (try_mark_partial_variable(deref_var->var, ir->array_index))
            {
               /* We've now taken care of foo and j, but i might contain a
                * subexpression that accesses shader inputs.  So manually
                * visit i and then continue with the parent.
                */
               inner_array->array_index->accept(this);
               return visit_continue_with_parent;
            }
         }
      }
   } else if (ir_dereference_variable * const deref_var =
              ir->array->as_dereference_variable()) {
      /* ir => foo[i], where foo is a variable. */
      if (is_multiple_vertices(this->shader_stage, deref_var->var)) {
         /* foo is a geometry or tessellation shader input, so i is
          * the vertex, and we're accessing the entire input.
          */
         mark_whole_variable(deref_var->var);
         /* We've now taken care of foo, but i might contain a subexpression
          * that accesses shader inputs.  So manually visit i and then
          * continue with the parent.
          */
         ir->array_index->accept(this);
         return visit_continue_with_parent;
      } else if (is_shader_inout(deref_var->var)) {
         /* foo is a shader input/output, but not a geometry shader input,
          * so i is the part of the input we're accessing.
          */
         if (try_mark_partial_variable(deref_var->var, ir->array_index))
            return visit_continue_with_parent;
      }
   }

   /* The expression is something we don't recognize.  Just visit its
    * subexpressions.
    */
   return visit_continue;
}

ir_visitor_status
ir_set_program_inouts_visitor::visit_enter(ir_function_signature *ir)
{
   /* We don't want to descend into the function parameters and
    * consider them as shader inputs or outputs.
    */
   visit_list_elements(this, &ir->body);
   return visit_continue_with_parent;
}

ir_visitor_status
ir_set_program_inouts_visitor::visit_enter(ir_discard *)
{
   /* discards are only allowed in fragment shaders. */
   assert(this->shader_stage == MESA_SHADER_FRAGMENT);

   prog->info.fs.uses_discard = true;

   return visit_continue;
}

ir_visitor_status
ir_set_program_inouts_visitor::visit_enter(ir_texture *ir)
{
   if (ir->op == ir_tg4)
      prog->info.uses_texture_gather = true;
   return visit_continue;
}

void
do_set_program_inouts(exec_list *instructions, struct gl_program *prog,
                      gl_shader_stage shader_stage)
{
   ir_set_program_inouts_visitor v(prog, shader_stage);

   prog->info.inputs_read = 0;
   prog->info.outputs_written = 0;
   prog->SecondaryOutputsWritten = 0;
   prog->info.outputs_read = 0;
   prog->info.patch_inputs_read = 0;
   prog->info.patch_outputs_written = 0;
   prog->info.system_values_read = 0;
   if (shader_stage == MESA_SHADER_FRAGMENT) {
      prog->info.fs.uses_sample_qualifier = false;
      prog->info.fs.uses_discard = false;
   }
   visit_list_elements(&v, instructions);
}
