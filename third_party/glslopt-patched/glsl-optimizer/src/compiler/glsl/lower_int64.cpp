/*
 * Copyright © 2016 Intel Corporation
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
 * \file lower_int64.cpp
 *
 * Lower 64-bit operations to 32-bit operations.  Each 64-bit value is lowered
 * to a uvec2.  For each operation that can be lowered, there is a function
 * called __builtin_foo with the same number of parameters that takes uvec2
 * sources and produces uvec2 results.  An operation like
 *
 *     uint64_t(x) * uint64_t(y)
 *
 * becomes
 *
 *     packUint2x32(__builtin_umul64(unpackUint2x32(x), unpackUint2x32(y)));
 */

#include "main/macros.h"
#include "compiler/glsl_types.h"
#include "ir.h"
#include "ir_rvalue_visitor.h"
#include "ir_builder.h"
#include "ir_optimization.h"
#include "util/hash_table.h"
#include "builtin_functions.h"

typedef ir_function_signature *(*function_generator)(void *mem_ctx,
                                                     builtin_available_predicate avail);

using namespace ir_builder;

namespace lower_64bit {
void expand_source(ir_factory &, ir_rvalue *val, ir_variable **expanded_src);

ir_dereference_variable *compact_destination(ir_factory &,
                                             const glsl_type *type,
                                             ir_variable *result[4]);

ir_rvalue *lower_op_to_function_call(ir_instruction *base_ir,
                                     ir_expression *ir,
                                     ir_function_signature *callee);
};

using namespace lower_64bit;

namespace {

class lower_64bit_visitor : public ir_rvalue_visitor {
public:
   lower_64bit_visitor(void *mem_ctx, exec_list *instructions, unsigned lower)
      : progress(false), lower(lower),
        function_list(), added_functions(&function_list, mem_ctx)
   {
      functions = _mesa_hash_table_create(mem_ctx,
                                          _mesa_hash_string,
                                          _mesa_key_string_equal);

      foreach_in_list(ir_instruction, node, instructions) {
         ir_function *const f = node->as_function();

         if (f == NULL || strncmp(f->name, "__builtin_", 10) != 0)
            continue;

         add_function(f);
      }
   }

   ~lower_64bit_visitor()
   {
      _mesa_hash_table_destroy(functions, NULL);
   }

   void handle_rvalue(ir_rvalue **rvalue);

   void add_function(ir_function *f)
   {
      _mesa_hash_table_insert(functions, f->name, f);
   }

   ir_function *find_function(const char *name)
   {
      struct hash_entry *const entry =
         _mesa_hash_table_search(functions, name);

      return entry != NULL ? (ir_function *) entry->data : NULL;
   }

   bool progress;

private:
   unsigned lower; /** Bitfield of which operations to lower */

   /** Hashtable containing all of the known functions in the IR */
   struct hash_table *functions;

public:
   exec_list function_list;

private:
   ir_factory added_functions;

   ir_rvalue *handle_op(ir_expression *ir, const char *function_name,
                        function_generator generator);
};

} /* anonymous namespace */

/**
 * Determine if a particular type of lowering should occur
 */
#define lowering(x) (this->lower & x)

bool
lower_64bit_integer_instructions(exec_list *instructions,
                                 unsigned what_to_lower)
{
   if (instructions->is_empty())
      return false;

   ir_instruction *first_inst = (ir_instruction *) instructions->get_head_raw();
   void *const mem_ctx = ralloc_parent(first_inst);
   lower_64bit_visitor v(mem_ctx, instructions, what_to_lower);

   visit_list_elements(&v, instructions);

   if (v.progress && !v.function_list.is_empty()) {
      /* Move all of the nodes from function_list to the head if the incoming
       * instruction list.
       */
      exec_node *const after = &instructions->head_sentinel;
      exec_node *const before = instructions->head_sentinel.next;
      exec_node *const head = v.function_list.head_sentinel.next;
      exec_node *const tail = v.function_list.tail_sentinel.prev;

      before->next = head;
      head->prev = before;

      after->prev = tail;
      tail->next = after;
   }

   return v.progress;
}


/**
 * Expand individual 64-bit values to uvec2 values
 *
 * Each operation is in one of a few forms.
 *
 *     vector op vector
 *     vector op scalar
 *     scalar op vector
 *     scalar op scalar
 *
 * In the 'vector op vector' case, the two vectors must have the same size.
 * In a way, the 'scalar op scalar' form is special case of the 'vector op
 * vector' form.
 *
 * This method generates a new set of uvec2 values for each element of a
 * single operand.  If the operand is a scalar, the uvec2 is replicated
 * multiple times.  A value like
 *
 *     u64vec3(a) + u64vec3(b)
 *
 * becomes
 *
 *     u64vec3 tmp0 = u64vec3(a) + u64vec3(b);
 *     uvec2 tmp1 = unpackUint2x32(tmp0.x);
 *     uvec2 tmp2 = unpackUint2x32(tmp0.y);
 *     uvec2 tmp3 = unpackUint2x32(tmp0.z);
 *
 * and the returned operands array contains ir_variable pointers to
 *
 *     { tmp1, tmp2, tmp3, tmp1 }
 */
void
lower_64bit::expand_source(ir_factory &body,
                           ir_rvalue *val,
                           ir_variable **expanded_src)
{
   assert(val->type->is_integer_64());

   ir_variable *const temp = body.make_temp(val->type, "tmp");

   body.emit(assign(temp, val));

   const ir_expression_operation unpack_opcode =
      val->type->base_type == GLSL_TYPE_UINT64
      ? ir_unop_unpack_uint_2x32 : ir_unop_unpack_int_2x32;

   const glsl_type *const type =
      val->type->base_type == GLSL_TYPE_UINT64
      ? glsl_type::uvec2_type : glsl_type::ivec2_type;

   unsigned i;
   for (i = 0; i < val->type->vector_elements; i++) {
      expanded_src[i] = body.make_temp(type, "expanded_64bit_source");

      body.emit(assign(expanded_src[i],
                       expr(unpack_opcode, swizzle(temp, i, 1))));
   }

   for (/* empty */; i < 4; i++)
      expanded_src[i] = expanded_src[0];
}

/**
 * Convert a series of uvec2 results into a single 64-bit integer vector
 */
ir_dereference_variable *
lower_64bit::compact_destination(ir_factory &body,
                                 const glsl_type *type,
                                 ir_variable *result[4])
{
   const ir_expression_operation pack_opcode =
      type->base_type == GLSL_TYPE_UINT64
      ? ir_unop_pack_uint_2x32 : ir_unop_pack_int_2x32;

   ir_variable *const compacted_result =
      body.make_temp(type, "compacted_64bit_result");

   for (unsigned i = 0; i < type->vector_elements; i++) {
      body.emit(assign(compacted_result,
                       expr(pack_opcode, result[i]),
                       1U << i));
   }

   void *const mem_ctx = ralloc_parent(compacted_result);
   return new(mem_ctx) ir_dereference_variable(compacted_result);
}

ir_rvalue *
lower_64bit::lower_op_to_function_call(ir_instruction *base_ir,
                                       ir_expression *ir,
                                       ir_function_signature *callee)
{
   const unsigned num_operands = ir->num_operands;
   ir_variable *src[4][4];
   ir_variable *dst[4];
   void *const mem_ctx = ralloc_parent(ir);
   exec_list instructions;
   unsigned source_components = 0;
   const glsl_type *const result_type =
      ir->type->base_type == GLSL_TYPE_UINT64
      ? glsl_type::uvec2_type : glsl_type::ivec2_type;

   ir_factory body(&instructions, mem_ctx);

   for (unsigned i = 0; i < num_operands; i++) {
      expand_source(body, ir->operands[i], src[i]);

      if (ir->operands[i]->type->vector_elements > source_components)
         source_components = ir->operands[i]->type->vector_elements;
   }

   for (unsigned i = 0; i < source_components; i++) {
      dst[i] = body.make_temp(result_type, "expanded_64bit_result");

      exec_list parameters;

      for (unsigned j = 0; j < num_operands; j++)
         parameters.push_tail(new(mem_ctx) ir_dereference_variable(src[j][i]));

      ir_dereference_variable *const return_deref =
         new(mem_ctx) ir_dereference_variable(dst[i]);

      ir_call *const c = new(mem_ctx) ir_call(callee,
                                              return_deref,
                                              &parameters);

      body.emit(c);
   }

   ir_rvalue *const rv = compact_destination(body, ir->type, dst);

   /* Move all of the nodes from instructions between base_ir and the
    * instruction before it.
    */
   exec_node *const after = base_ir;
   exec_node *const before = after->prev;
   exec_node *const head = instructions.head_sentinel.next;
   exec_node *const tail = instructions.tail_sentinel.prev;

   before->next = head;
   head->prev = before;

   after->prev = tail;
   tail->next = after;

   return rv;
}

ir_rvalue *
lower_64bit_visitor::handle_op(ir_expression *ir,
                               const char *function_name,
                               function_generator generator)
{
   for (unsigned i = 0; i < ir->num_operands; i++)
      if (!ir->operands[i]->type->is_integer_64())
         return ir;

   /* Get a handle to the correct ir_function_signature for the core
    * operation.
    */
   ir_function_signature *callee = NULL;
   ir_function *f = find_function(function_name);

   if (f != NULL) {
      callee = (ir_function_signature *) f->signatures.get_head();
      assert(callee != NULL && callee->ir_type == ir_type_function_signature);
   } else {
      f = new(base_ir) ir_function(function_name);
      callee = generator(base_ir, NULL);

      f->add_signature(callee);

      add_function(f);
   }

   this->progress = true;
   return lower_op_to_function_call(this->base_ir, ir, callee);
}

void
lower_64bit_visitor::handle_rvalue(ir_rvalue **rvalue)
{
   if (*rvalue == NULL || (*rvalue)->ir_type != ir_type_expression)
      return;

   ir_expression *const ir = (*rvalue)->as_expression();
   assert(ir != NULL);

   switch (ir->operation) {
   case ir_unop_sign:
      if (lowering(SIGN64)) {
         *rvalue = handle_op(ir, "__builtin_sign64", generate_ir::sign64);
      }
      break;

   case ir_binop_div:
      if (lowering(DIV64)) {
         if (ir->type->base_type == GLSL_TYPE_UINT64) {
            *rvalue = handle_op(ir, "__builtin_udiv64", generate_ir::udiv64);
         } else {
            *rvalue = handle_op(ir, "__builtin_idiv64", generate_ir::idiv64);
         }
      }
      break;

   case ir_binop_mod:
      if (lowering(MOD64)) {
         if (ir->type->base_type == GLSL_TYPE_UINT64) {
            *rvalue = handle_op(ir, "__builtin_umod64", generate_ir::umod64);
         } else {
            *rvalue = handle_op(ir, "__builtin_imod64", generate_ir::imod64);
         }
      }
      break;

   case ir_binop_mul:
      if (lowering(MUL64)) {
         *rvalue = handle_op(ir, "__builtin_umul64", generate_ir::umul64);
      }
      break;

   default:
      break;
   }
}
