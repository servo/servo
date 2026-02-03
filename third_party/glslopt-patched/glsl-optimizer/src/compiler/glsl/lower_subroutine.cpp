/*
 * Copyright © 2015 Red Hat
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
 * \file lower_subroutine.cpp
 *
 * lowers subroutines to an if ladder.
 */

#include "compiler/glsl_types.h"
#include "glsl_parser_extras.h"
#include "ir.h"
#include "ir_builder.h"

using namespace ir_builder;
namespace {

class lower_subroutine_visitor : public ir_hierarchical_visitor {
public:
   lower_subroutine_visitor(struct _mesa_glsl_parse_state *state)
      : state(state)
   {
      this->progress = false;
   }

   ir_visitor_status visit_leave(ir_call *);
   ir_call *call_clone(ir_call *call, ir_function_signature *callee);
   bool progress;
   struct _mesa_glsl_parse_state *state;
};

}

bool
lower_subroutine(exec_list *instructions, struct _mesa_glsl_parse_state *state)
{
   lower_subroutine_visitor v(state);
   visit_list_elements(&v, instructions);
   return v.progress;
}

ir_call *
lower_subroutine_visitor::call_clone(ir_call *call, ir_function_signature *callee)
{
   void *mem_ctx = ralloc_parent(call);
   ir_dereference_variable *new_return_ref = NULL;
   if (call->return_deref != NULL)
      new_return_ref = call->return_deref->clone(mem_ctx, NULL);

   exec_list new_parameters;

   foreach_in_list(ir_instruction, ir, &call->actual_parameters) {
      new_parameters.push_tail(ir->clone(mem_ctx, NULL));
   }

   return new(mem_ctx) ir_call(callee, new_return_ref, &new_parameters);
}

ir_visitor_status
lower_subroutine_visitor::visit_leave(ir_call *ir)
{
   if (!ir->sub_var)
      return visit_continue;

   void *mem_ctx = ralloc_parent(ir);
   ir_if *last_branch = NULL;

   for (int s = this->state->num_subroutines - 1; s >= 0; s--) {
      ir_rvalue *var;
      ir_function *fn = this->state->subroutines[s];
      ir_constant *lc = new(mem_ctx)ir_constant(fn->subroutine_index);

      bool is_compat = false;

      for (int i = 0; i < fn->num_subroutine_types; i++) {
         if (ir->sub_var->type->without_array() == fn->subroutine_types[i]) {
            is_compat = true;
            break;
         }
      }
      if (is_compat == false)
         continue;

      if (ir->array_idx != NULL)
         var = ir->array_idx->clone(mem_ctx, NULL);
      else
         var = new(mem_ctx) ir_dereference_variable(ir->sub_var);

      ir_function_signature *sub_sig =
         fn->exact_matching_signature(this->state,
                                      &ir->actual_parameters);

      ir_call *new_call = call_clone(ir, sub_sig);
      if (!last_branch)
         last_branch = if_tree(equal(subr_to_int(var), lc), new_call);
      else
         last_branch = if_tree(equal(subr_to_int(var), lc), new_call, last_branch);
   }
   if (last_branch)
      ir->insert_before(last_branch);
   ir->remove();

   return visit_continue;
}
