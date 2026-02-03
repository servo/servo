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
 * \file ir_rvalue_visitor.cpp
 *
 * Generic class to implement the common pattern we have of wanting to
 * visit each ir_rvalue * and possibly change that node to a different
 * class.
 */

#include "ir.h"
#include "ir_visitor.h"
#include "ir_rvalue_visitor.h"
#include "compiler/glsl_types.h"

ir_visitor_status
ir_rvalue_base_visitor::rvalue_visit(ir_expression *ir)
{
   unsigned int operand;

   for (operand = 0; operand < ir->num_operands; operand++) {
      handle_rvalue(&ir->operands[operand]);
   }

   return visit_continue;
}

ir_visitor_status
ir_rvalue_base_visitor::rvalue_visit(ir_texture *ir)
{
   handle_rvalue(&ir->coordinate);
   handle_rvalue(&ir->projector);
   handle_rvalue(&ir->shadow_comparator);
   handle_rvalue(&ir->offset);

   switch (ir->op) {
   case ir_tex:
   case ir_lod:
   case ir_query_levels:
   case ir_texture_samples:
   case ir_samples_identical:
      break;
   case ir_txb:
      handle_rvalue(&ir->lod_info.bias);
      break;
   case ir_txf:
   case ir_txl:
   case ir_txs:
      handle_rvalue(&ir->lod_info.lod);
      break;
   case ir_txf_ms:
      handle_rvalue(&ir->lod_info.sample_index);
      break;
   case ir_txd:
      handle_rvalue(&ir->lod_info.grad.dPdx);
      handle_rvalue(&ir->lod_info.grad.dPdy);
      break;
   case ir_tg4:
      handle_rvalue(&ir->lod_info.component);
      break;
   }

   return visit_continue;
}

ir_visitor_status
ir_rvalue_base_visitor::rvalue_visit(ir_swizzle *ir)
{
   handle_rvalue(&ir->val);
   return visit_continue;
}

ir_visitor_status
ir_rvalue_base_visitor::rvalue_visit(ir_dereference_array *ir)
{
   /* The array index is not the target of the assignment, so clear the
    * 'in_assignee' flag.  Restore it after returning from the array index.
    */
   const bool was_in_assignee = this->in_assignee;
   this->in_assignee = false;
   handle_rvalue(&ir->array_index);
   this->in_assignee = was_in_assignee;

   handle_rvalue(&ir->array);
   return visit_continue;
}

ir_visitor_status
ir_rvalue_base_visitor::rvalue_visit(ir_dereference_record *ir)
{
   handle_rvalue(&ir->record);
   return visit_continue;
}

ir_visitor_status
ir_rvalue_base_visitor::rvalue_visit(ir_assignment *ir)
{
   handle_rvalue(&ir->rhs);
   handle_rvalue(&ir->condition);

   return visit_continue;
}

ir_visitor_status
ir_rvalue_base_visitor::rvalue_visit(ir_call *ir)
{
   foreach_in_list_safe(ir_rvalue, param, &ir->actual_parameters) {
      ir_rvalue *new_param = param;
      handle_rvalue(&new_param);

      if (new_param != param) {
	 param->replace_with(new_param);
      }
   }
   return visit_continue;
}

ir_visitor_status
ir_rvalue_base_visitor::rvalue_visit(ir_discard *ir)
{
   handle_rvalue(&ir->condition);
   return visit_continue;
}

ir_visitor_status
ir_rvalue_base_visitor::rvalue_visit(ir_return *ir)
{
   handle_rvalue(&ir->value);
   return visit_continue;
}

ir_visitor_status
ir_rvalue_base_visitor::rvalue_visit(ir_if *ir)
{
   handle_rvalue(&ir->condition);
   return visit_continue;
}

ir_visitor_status
ir_rvalue_base_visitor::rvalue_visit(ir_emit_vertex *ir)
{
   handle_rvalue(&ir->stream);
   return visit_continue;
}

ir_visitor_status
ir_rvalue_base_visitor::rvalue_visit(ir_end_primitive *ir)
{
   handle_rvalue(&ir->stream);
   return visit_continue;
}

ir_visitor_status
ir_rvalue_visitor::visit_leave(ir_expression *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_visitor::visit_leave(ir_texture *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_visitor::visit_leave(ir_swizzle *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_visitor::visit_leave(ir_dereference_array *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_visitor::visit_leave(ir_dereference_record *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_visitor::visit_leave(ir_assignment *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_visitor::visit_leave(ir_call *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_visitor::visit_leave(ir_discard *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_visitor::visit_leave(ir_return *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_visitor::visit_leave(ir_if *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_visitor::visit_leave(ir_emit_vertex *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_visitor::visit_leave(ir_end_primitive *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_enter_visitor::visit_enter(ir_expression *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_enter_visitor::visit_enter(ir_texture *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_enter_visitor::visit_enter(ir_swizzle *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_enter_visitor::visit_enter(ir_dereference_array *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_enter_visitor::visit_enter(ir_dereference_record *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_enter_visitor::visit_enter(ir_assignment *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_enter_visitor::visit_enter(ir_call *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_enter_visitor::visit_enter(ir_discard *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_enter_visitor::visit_enter(ir_return *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_enter_visitor::visit_enter(ir_if *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_enter_visitor::visit_enter(ir_emit_vertex *ir)
{
   return rvalue_visit(ir);
}

ir_visitor_status
ir_rvalue_enter_visitor::visit_enter(ir_end_primitive *ir)
{
   return rvalue_visit(ir);
}
