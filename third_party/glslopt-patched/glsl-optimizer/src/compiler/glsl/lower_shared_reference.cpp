/*
 * Copyright (c) 2015 Intel Corporation
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
 * \file lower_shared_reference.cpp
 *
 * IR lower pass to replace dereferences of compute shader shared variables
 * with intrinsic function calls.
 *
 * This relieves drivers of the responsibility of allocating space for the
 * shared variables in the shared memory region.
 */

#include "lower_buffer_access.h"
#include "ir_builder.h"
#include "linker.h"
#include "main/macros.h"
#include "util/list.h"
#include "glsl_parser_extras.h"
#include "main/mtypes.h"

using namespace ir_builder;

namespace {

struct var_offset {
   struct list_head node;
   const ir_variable *var;
   unsigned offset;
};

class lower_shared_reference_visitor :
      public lower_buffer_access::lower_buffer_access {
public:

   lower_shared_reference_visitor(struct gl_linked_shader *shader)
      : list_ctx(ralloc_context(NULL)), shader(shader), shared_size(0u)
   {
      list_inithead(&var_offsets);
   }

   ~lower_shared_reference_visitor()
   {
      ralloc_free(list_ctx);
   }

   enum {
      shared_load_access,
      shared_store_access,
      shared_atomic_access,
   } buffer_access_type;

   void insert_buffer_access(void *mem_ctx, ir_dereference *deref,
                             const glsl_type *type, ir_rvalue *offset,
                             unsigned mask, int channel);

   void handle_rvalue(ir_rvalue **rvalue);
   ir_visitor_status visit_enter(ir_assignment *ir);
   void handle_assignment(ir_assignment *ir);

   ir_call *lower_shared_atomic_intrinsic(ir_call *ir);
   ir_call *check_for_shared_atomic_intrinsic(ir_call *ir);
   ir_visitor_status visit_enter(ir_call *ir);

   unsigned get_shared_offset(const ir_variable *);

   ir_call *shared_load(void *mem_ctx, const struct glsl_type *type,
                        ir_rvalue *offset);
   ir_call *shared_store(void *mem_ctx, ir_rvalue *deref, ir_rvalue *offset,
                         unsigned write_mask);

   void *list_ctx;
   struct gl_linked_shader *shader;
   struct list_head var_offsets;
   unsigned shared_size;
   bool progress;
};

unsigned
lower_shared_reference_visitor::get_shared_offset(const ir_variable *var)
{
   list_for_each_entry(var_offset, var_entry, &var_offsets, node) {
      if (var_entry->var == var)
         return var_entry->offset;
   }

   struct var_offset *new_entry = rzalloc(list_ctx, struct var_offset);
   list_add(&new_entry->node, &var_offsets);
   new_entry->var = var;

   unsigned var_align = var->type->std430_base_alignment(false);
   new_entry->offset = glsl_align(shared_size, var_align);

   unsigned var_size = var->type->std430_size(false);
   shared_size = new_entry->offset + var_size;

   return new_entry->offset;
}

void
lower_shared_reference_visitor::handle_rvalue(ir_rvalue **rvalue)
{
   if (!*rvalue)
      return;

   ir_dereference *deref = (*rvalue)->as_dereference();
   if (!deref)
      return;

   ir_variable *var = deref->variable_referenced();
   if (!var || var->data.mode != ir_var_shader_shared)
      return;

   buffer_access_type = shared_load_access;

   void *mem_ctx = ralloc_parent(shader->ir);

   ir_rvalue *offset = NULL;
   unsigned const_offset = get_shared_offset(var);
   bool row_major;
   const glsl_type *matrix_type;
   assert(var->get_interface_type() == NULL);
   const enum glsl_interface_packing packing = GLSL_INTERFACE_PACKING_STD430;

   setup_buffer_access(mem_ctx, deref,
                       &offset, &const_offset,
                       &row_major, &matrix_type, NULL, packing);

   /* Now that we've calculated the offset to the start of the
    * dereference, walk over the type and emit loads into a temporary.
    */
   const glsl_type *type = (*rvalue)->type;
   ir_variable *load_var = new(mem_ctx) ir_variable(type,
                                                    "shared_load_temp",
                                                    ir_var_temporary);
   base_ir->insert_before(load_var);

   ir_variable *load_offset = new(mem_ctx) ir_variable(glsl_type::uint_type,
                                                       "shared_load_temp_offset",
                                                       ir_var_temporary);
   base_ir->insert_before(load_offset);
   base_ir->insert_before(assign(load_offset, offset));

   deref = new(mem_ctx) ir_dereference_variable(load_var);

   emit_access(mem_ctx, false, deref, load_offset, const_offset, row_major,
               matrix_type, packing, 0);

   *rvalue = deref;

   progress = true;
}

void
lower_shared_reference_visitor::handle_assignment(ir_assignment *ir)
{
   if (!ir || !ir->lhs)
      return;

   ir_rvalue *rvalue = ir->lhs->as_rvalue();
   if (!rvalue)
      return;

   ir_dereference *deref = ir->lhs->as_dereference();
   if (!deref)
      return;

   ir_variable *var = ir->lhs->variable_referenced();
   if (!var || var->data.mode != ir_var_shader_shared)
      return;

   buffer_access_type = shared_store_access;

   /* We have a write to a shared variable, so declare a temporary and rewrite
    * the assignment so that the temporary is the LHS.
    */
   void *mem_ctx = ralloc_parent(shader->ir);

   const glsl_type *type = rvalue->type;
   ir_variable *store_var = new(mem_ctx) ir_variable(type,
                                                     "shared_store_temp",
                                                     ir_var_temporary);
   base_ir->insert_before(store_var);
   ir->lhs = new(mem_ctx) ir_dereference_variable(store_var);

   ir_rvalue *offset = NULL;
   unsigned const_offset = get_shared_offset(var);
   bool row_major;
   const glsl_type *matrix_type;
   assert(var->get_interface_type() == NULL);
   const enum glsl_interface_packing packing = GLSL_INTERFACE_PACKING_STD430;

   setup_buffer_access(mem_ctx, deref,
                       &offset, &const_offset,
                       &row_major, &matrix_type, NULL, packing);

   deref = new(mem_ctx) ir_dereference_variable(store_var);

   ir_variable *store_offset = new(mem_ctx) ir_variable(glsl_type::uint_type,
                                                        "shared_store_temp_offset",
                                                        ir_var_temporary);
   base_ir->insert_before(store_offset);
   base_ir->insert_before(assign(store_offset, offset));

   /* Now we have to write the value assigned to the temporary back to memory */
   emit_access(mem_ctx, true, deref, store_offset, const_offset, row_major,
               matrix_type, packing, ir->write_mask);

   progress = true;
}

ir_visitor_status
lower_shared_reference_visitor::visit_enter(ir_assignment *ir)
{
   handle_assignment(ir);
   return rvalue_visit(ir);
}

void
lower_shared_reference_visitor::insert_buffer_access(void *mem_ctx,
                                                     ir_dereference *deref,
                                                     const glsl_type *type,
                                                     ir_rvalue *offset,
                                                     unsigned mask,
                                                     int /* channel */)
{
   if (buffer_access_type == shared_store_access) {
      ir_call *store = shared_store(mem_ctx, deref, offset, mask);
      base_ir->insert_after(store);
   } else {
      ir_call *load = shared_load(mem_ctx, type, offset);
      base_ir->insert_before(load);
      ir_rvalue *value = load->return_deref->as_rvalue()->clone(mem_ctx, NULL);
      base_ir->insert_before(assign(deref->clone(mem_ctx, NULL),
                                    value));
   }
}

static bool
compute_shader_enabled(const _mesa_glsl_parse_state *state)
{
   return state->stage == MESA_SHADER_COMPUTE;
}

ir_call *
lower_shared_reference_visitor::shared_store(void *mem_ctx,
                                             ir_rvalue *deref,
                                             ir_rvalue *offset,
                                             unsigned write_mask)
{
   exec_list sig_params;

   ir_variable *offset_ref = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "offset" , ir_var_function_in);
   sig_params.push_tail(offset_ref);

   ir_variable *val_ref = new(mem_ctx)
      ir_variable(deref->type, "value" , ir_var_function_in);
   sig_params.push_tail(val_ref);

   ir_variable *writemask_ref = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "write_mask" , ir_var_function_in);
   sig_params.push_tail(writemask_ref);

   ir_function_signature *sig = new(mem_ctx)
      ir_function_signature(glsl_type::void_type, compute_shader_enabled);
   assert(sig);
   sig->replace_parameters(&sig_params);
   sig->intrinsic_id = ir_intrinsic_shared_store;

   ir_function *f = new(mem_ctx) ir_function("__intrinsic_store_shared");
   f->add_signature(sig);

   exec_list call_params;
   call_params.push_tail(offset->clone(mem_ctx, NULL));
   call_params.push_tail(deref->clone(mem_ctx, NULL));
   call_params.push_tail(new(mem_ctx) ir_constant(write_mask));
   return new(mem_ctx) ir_call(sig, NULL, &call_params);
}

ir_call *
lower_shared_reference_visitor::shared_load(void *mem_ctx,
                                            const struct glsl_type *type,
                                            ir_rvalue *offset)
{
   exec_list sig_params;

   ir_variable *offset_ref = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "offset_ref" , ir_var_function_in);
   sig_params.push_tail(offset_ref);

   ir_function_signature *sig =
      new(mem_ctx) ir_function_signature(type, compute_shader_enabled);
   assert(sig);
   sig->replace_parameters(&sig_params);
   sig->intrinsic_id = ir_intrinsic_shared_load;

   ir_function *f = new(mem_ctx) ir_function("__intrinsic_load_shared");
   f->add_signature(sig);

   ir_variable *result = new(mem_ctx)
      ir_variable(type, "shared_load_result", ir_var_temporary);
   base_ir->insert_before(result);
   ir_dereference_variable *deref_result = new(mem_ctx)
      ir_dereference_variable(result);

   exec_list call_params;
   call_params.push_tail(offset->clone(mem_ctx, NULL));

   return new(mem_ctx) ir_call(sig, deref_result, &call_params);
}

/* Lowers the intrinsic call to a new internal intrinsic that swaps the access
 * to the shared variable in the first parameter by an offset. This involves
 * creating the new internal intrinsic (i.e. the new function signature).
 */
ir_call *
lower_shared_reference_visitor::lower_shared_atomic_intrinsic(ir_call *ir)
{
   /* Shared atomics usually have 2 parameters, the shared variable and an
    * integer argument. The exception is CompSwap, that has an additional
    * integer parameter.
    */
   int param_count = ir->actual_parameters.length();
   assert(param_count == 2 || param_count == 3);

   /* First argument must be a scalar integer shared variable */
   exec_node *param = ir->actual_parameters.get_head();
   ir_instruction *inst = (ir_instruction *) param;
   assert(inst->ir_type == ir_type_dereference_variable ||
          inst->ir_type == ir_type_dereference_array ||
          inst->ir_type == ir_type_dereference_record ||
          inst->ir_type == ir_type_swizzle);

   ir_rvalue *deref = (ir_rvalue *) inst;
   assert(deref->type->is_scalar() &&
          (deref->type->is_integer_32() || deref->type->is_float()));

   ir_variable *var = deref->variable_referenced();
   assert(var);

   /* Compute the offset to the start if the dereference
    */
   void *mem_ctx = ralloc_parent(shader->ir);

   ir_rvalue *offset = NULL;
   unsigned const_offset = get_shared_offset(var);
   bool row_major;
   const glsl_type *matrix_type;
   assert(var->get_interface_type() == NULL);
   const enum glsl_interface_packing packing = GLSL_INTERFACE_PACKING_STD430;
   buffer_access_type = shared_atomic_access;

   setup_buffer_access(mem_ctx, deref,
                       &offset, &const_offset,
                       &row_major, &matrix_type, NULL, packing);

   assert(offset);
   assert(!row_major);
   assert(matrix_type == NULL);

   ir_rvalue *deref_offset =
      add(offset, new(mem_ctx) ir_constant(const_offset));

   /* Create the new internal function signature that will take an offset
    * instead of a shared variable
    */
   exec_list sig_params;
   ir_variable *sig_param = new(mem_ctx)
      ir_variable(glsl_type::uint_type, "offset" , ir_var_function_in);
   sig_params.push_tail(sig_param);

   const glsl_type *type = deref->type->get_scalar_type();
   sig_param = new(mem_ctx)
         ir_variable(type, "data1", ir_var_function_in);
   sig_params.push_tail(sig_param);

   if (param_count == 3) {
      sig_param = new(mem_ctx)
            ir_variable(type, "data2", ir_var_function_in);
      sig_params.push_tail(sig_param);
   }

   ir_function_signature *sig =
      new(mem_ctx) ir_function_signature(deref->type,
                                         compute_shader_enabled);
   assert(sig);
   sig->replace_parameters(&sig_params);

   assert(ir->callee->intrinsic_id >= ir_intrinsic_generic_load);
   assert(ir->callee->intrinsic_id <= ir_intrinsic_generic_atomic_comp_swap);
   sig->intrinsic_id = MAP_INTRINSIC_TO_TYPE(ir->callee->intrinsic_id, shared);

   char func_name[64];
   sprintf(func_name, "%s_shared", ir->callee_name());
   ir_function *f = new(mem_ctx) ir_function(func_name);
   f->add_signature(sig);

   /* Now, create the call to the internal intrinsic */
   exec_list call_params;
   call_params.push_tail(deref_offset);
   param = ir->actual_parameters.get_head()->get_next();
   ir_rvalue *param_as_rvalue = ((ir_instruction *) param)->as_rvalue();
   call_params.push_tail(param_as_rvalue->clone(mem_ctx, NULL));
   if (param_count == 3) {
      param = param->get_next();
      param_as_rvalue = ((ir_instruction *) param)->as_rvalue();
      call_params.push_tail(param_as_rvalue->clone(mem_ctx, NULL));
   }
   ir_dereference_variable *return_deref =
      ir->return_deref->clone(mem_ctx, NULL);
   return new(mem_ctx) ir_call(sig, return_deref, &call_params);
}

ir_call *
lower_shared_reference_visitor::check_for_shared_atomic_intrinsic(ir_call *ir)
{
   exec_list& params = ir->actual_parameters;

   if (params.length() < 2 || params.length() > 3)
      return ir;

   ir_rvalue *rvalue =
      ((ir_instruction *) params.get_head())->as_rvalue();
   if (!rvalue)
      return ir;

   ir_variable *var = rvalue->variable_referenced();
   if (!var || var->data.mode != ir_var_shader_shared)
      return ir;

   const enum ir_intrinsic_id id = ir->callee->intrinsic_id;
   if (id == ir_intrinsic_generic_atomic_add ||
       id == ir_intrinsic_generic_atomic_min ||
       id == ir_intrinsic_generic_atomic_max ||
       id == ir_intrinsic_generic_atomic_and ||
       id == ir_intrinsic_generic_atomic_or ||
       id == ir_intrinsic_generic_atomic_xor ||
       id == ir_intrinsic_generic_atomic_exchange ||
       id == ir_intrinsic_generic_atomic_comp_swap) {
      return lower_shared_atomic_intrinsic(ir);
   }

   return ir;
}

ir_visitor_status
lower_shared_reference_visitor::visit_enter(ir_call *ir)
{
   ir_call *new_ir = check_for_shared_atomic_intrinsic(ir);
   if (new_ir != ir) {
      progress = true;
      base_ir->replace_with(new_ir);
      return visit_continue_with_parent;
   }

   return rvalue_visit(ir);
}

} /* unnamed namespace */

void
lower_shared_reference(struct gl_context *ctx,
                       struct gl_shader_program *prog,
                       struct gl_linked_shader *shader)
{
   if (shader->Stage != MESA_SHADER_COMPUTE)
      return;

   lower_shared_reference_visitor v(shader);

   /* Loop over the instructions lowering references, because we take a deref
    * of an shared variable array using a shared variable dereference as the
    * index will produce a collection of instructions all of which have cloned
    * shared variable dereferences for that array index.
    */
   do {
      v.progress = false;
      visit_list_elements(&v, shader->ir);
   } while (v.progress);

   prog->Comp.SharedSize = v.shared_size;

   /* Section 19.1 (Compute Shader Variables) of the OpenGL 4.5 (Core Profile)
    * specification says:
    *
    *   "There is a limit to the total size of all variables declared as
    *    shared in a single program object. This limit, expressed in units of
    *    basic machine units, may be queried as the value of
    *    MAX_COMPUTE_SHARED_MEMORY_SIZE."
    */
   if (prog->Comp.SharedSize > ctx->Const.MaxComputeSharedMemorySize) {
      linker_error(prog, "Too much shared memory used (%u/%u)\n",
                   prog->Comp.SharedSize,
                   ctx->Const.MaxComputeSharedMemorySize);
   }
}
