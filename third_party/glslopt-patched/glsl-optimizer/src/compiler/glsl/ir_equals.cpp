/*
 * Copyright © 2013 Intel Corporation
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

#include "ir.h"

/**
 * Helper for checking equality when one instruction might be NULL, since you
 * can't access a's vtable in that case.
 */
static bool
possibly_null_equals(const ir_instruction *a, const ir_instruction *b,
                     enum ir_node_type ignore)
{
   if (!a || !b)
      return !a && !b;

   return a->equals(b, ignore);
}

/**
 * The base equality function: Return not equal for anything we don't know
 * about.
 */
bool
ir_instruction::equals(const ir_instruction *, enum ir_node_type) const
{
   return false;
}

bool
ir_constant::equals(const ir_instruction *ir, enum ir_node_type) const
{
   const ir_constant *other = ir->as_constant();
   if (!other)
      return false;

   if (type != other->type)
      return false;

   for (unsigned i = 0; i < type->components(); i++) {
      if (type->is_double()) {
         if (value.d[i] != other->value.d[i])
            return false;
      } else {
         if (value.u[i] != other->value.u[i])
            return false;
      }
   }

   return true;
}

bool
ir_dereference_variable::equals(const ir_instruction *ir,
                                enum ir_node_type) const
{
   const ir_dereference_variable *other = ir->as_dereference_variable();
   if (!other)
      return false;

   return var == other->var;
}

bool
ir_dereference_array::equals(const ir_instruction *ir,
                             enum ir_node_type ignore) const
{
   const ir_dereference_array *other = ir->as_dereference_array();
   if (!other)
      return false;

   if (type != other->type)
      return false;

   if (!array->equals(other->array, ignore))
      return false;

   if (!array_index->equals(other->array_index, ignore))
      return false;

   return true;
}

bool
ir_swizzle::equals(const ir_instruction *ir,
                   enum ir_node_type ignore) const
{
   const ir_swizzle *other = ir->as_swizzle();
   if (!other)
      return false;

   if (type != other->type)
      return false;

   if (ignore != ir_type_swizzle) {
      if (mask.x != other->mask.x ||
          mask.y != other->mask.y ||
          mask.z != other->mask.z ||
          mask.w != other->mask.w) {
         return false;
      }
   }

   return val->equals(other->val, ignore);
}

bool
ir_texture::equals(const ir_instruction *ir, enum ir_node_type ignore) const
{
   const ir_texture *other = ir->as_texture();
   if (!other)
      return false;

   if (type != other->type)
      return false;

   if (op != other->op)
      return false;

   if (!possibly_null_equals(coordinate, other->coordinate, ignore))
      return false;

   if (!possibly_null_equals(projector, other->projector, ignore))
      return false;

   if (!possibly_null_equals(shadow_comparator, other->shadow_comparator, ignore))
      return false;

   if (!possibly_null_equals(offset, other->offset, ignore))
      return false;

   if (!sampler->equals(other->sampler, ignore))
      return false;

   switch (op) {
   case ir_tex:
   case ir_lod:
   case ir_query_levels:
   case ir_texture_samples:
   case ir_samples_identical:
      break;
   case ir_txb:
      if (!lod_info.bias->equals(other->lod_info.bias, ignore))
         return false;
      break;
   case ir_txl:
   case ir_txf:
   case ir_txs:
      if (!lod_info.lod->equals(other->lod_info.lod, ignore))
         return false;
      break;
   case ir_txd:
      if (!lod_info.grad.dPdx->equals(other->lod_info.grad.dPdx, ignore) ||
          !lod_info.grad.dPdy->equals(other->lod_info.grad.dPdy, ignore))
         return false;
      break;
   case ir_txf_ms:
      if (!lod_info.sample_index->equals(other->lod_info.sample_index, ignore))
         return false;
      break;
   case ir_tg4:
      if (!lod_info.component->equals(other->lod_info.component, ignore))
         return false;
      break;
   default:
      assert(!"Unrecognized texture op");
   }

   return true;
}

bool
ir_expression::equals(const ir_instruction *ir, enum ir_node_type ignore) const
{
   const ir_expression *other = ir->as_expression();
   if (!other)
      return false;

   if (type != other->type)
      return false;

   if (operation != other->operation)
      return false;

   for (unsigned i = 0; i < num_operands; i++) {
      if (!operands[i]->equals(other->operands[i], ignore))
         return false;
   }

   return true;
}
