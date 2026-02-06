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

#include <inttypes.h> /* for PRIx64 macro */
#include "ir.h"
#include "ir_hierarchical_visitor.h"
#include "ir_builder_print_visitor.h"
#include "compiler/glsl_types.h"
#include "glsl_parser_extras.h"
#include "main/macros.h"
#include "util/hash_table.h"
#include "util/u_string.h"

class ir_builder_print_visitor : public ir_hierarchical_visitor {
public:
   ir_builder_print_visitor(FILE *f);
   virtual ~ir_builder_print_visitor();

   void indent(void);

   virtual ir_visitor_status visit(class ir_variable *);
   virtual ir_visitor_status visit(class ir_dereference_variable *);
   virtual ir_visitor_status visit(class ir_constant *);
   virtual ir_visitor_status visit(class ir_loop_jump *);

   virtual ir_visitor_status visit_enter(class ir_if *);

   virtual ir_visitor_status visit_enter(class ir_loop *);
   virtual ir_visitor_status visit_leave(class ir_loop *);

   virtual ir_visitor_status visit_enter(class ir_function_signature *);
   virtual ir_visitor_status visit_leave(class ir_function_signature *);

   virtual ir_visitor_status visit_enter(class ir_expression *);

   virtual ir_visitor_status visit_enter(class ir_assignment *);
   virtual ir_visitor_status visit_leave(class ir_assignment *);

   virtual ir_visitor_status visit_leave(class ir_call *);
   virtual ir_visitor_status visit_leave(class ir_swizzle *);
   virtual ir_visitor_status visit_leave(class ir_return *);

   virtual ir_visitor_status visit_enter(ir_texture *ir);

private:
   void print_with_indent(const char *fmt, ...);
   void print_without_indent(const char *fmt, ...);

   void print_without_declaration(const ir_rvalue *ir);
   void print_without_declaration(const ir_constant *ir);
   void print_without_declaration(const ir_dereference_variable *ir);
   void print_without_declaration(const ir_swizzle *ir);
   void print_without_declaration(const ir_expression *ir);

   unsigned next_ir_index;

   /**
    * Mapping from ir_instruction * -> index used in the generated C code
    * variable name.
    */
   hash_table *index_map;

   FILE *f;

   int indentation;
};

/* An operand is "simple" if it can be compactly printed on one line.
 */
static bool
is_simple_operand(const ir_rvalue *ir, unsigned depth = 1)
{
   if (depth == 0)
      return false;

   switch (ir->ir_type) {
   case ir_type_dereference_variable:
      return true;

   case ir_type_constant: {
      if (ir->type == glsl_type::uint_type ||
          ir->type == glsl_type::int_type ||
          ir->type == glsl_type::float_type ||
          ir->type == glsl_type::bool_type)
         return true;

      const ir_constant *const c = (ir_constant *) ir;
      ir_constant_data all_zero;
      memset(&all_zero, 0, sizeof(all_zero));

      return memcmp(&c->value, &all_zero, sizeof(all_zero)) == 0;
   }

   case ir_type_swizzle: {
      const ir_swizzle *swiz = (ir_swizzle *) ir;
      return swiz->mask.num_components == 1 &&
             is_simple_operand(swiz->val, depth);
   }

   case ir_type_expression: {
      const ir_expression *expr = (ir_expression *) ir;

      for (unsigned i = 0; i < expr->num_operands; i++) {
         if (!is_simple_operand(expr->operands[i], depth - 1))
            return false;
      }

      return true;
   }

   default:
      return false;
   }
}

void
_mesa_print_builder_for_ir(FILE *f, exec_list *instructions)
{
   ir_builder_print_visitor v(f);
   v.run(instructions);
}

ir_builder_print_visitor::ir_builder_print_visitor(FILE *f)
   : next_ir_index(1), f(f), indentation(0)
{
   index_map = _mesa_pointer_hash_table_create(NULL);
}

ir_builder_print_visitor::~ir_builder_print_visitor()
{
   _mesa_hash_table_destroy(index_map, NULL);
}

void ir_builder_print_visitor::indent(void)
{
   for (int i = 0; i < indentation; i++)
      fprintf(f, "   ");
}

void
ir_builder_print_visitor::print_with_indent(const char *fmt, ...)
{
   va_list ap;

   indent();

   va_start(ap, fmt);
   vfprintf(f, fmt, ap);
   va_end(ap);
}

void
ir_builder_print_visitor::print_without_indent(const char *fmt, ...)
{
   va_list ap;

   va_start(ap, fmt);
   vfprintf(f, fmt, ap);
   va_end(ap);
}

void
ir_builder_print_visitor::print_without_declaration(const ir_rvalue *ir)
{
   switch (ir->ir_type) {
   case ir_type_dereference_variable:
      print_without_declaration((ir_dereference_variable *) ir);
      break;
   case ir_type_constant:
      print_without_declaration((ir_constant *) ir);
      break;
   case ir_type_swizzle:
      print_without_declaration((ir_swizzle *) ir);
      break;
   case ir_type_expression:
      print_without_declaration((ir_expression *) ir);
      break;
   default:
      unreachable("Invalid IR type.");
   }
}

ir_visitor_status
ir_builder_print_visitor::visit(ir_variable *ir)
{
   const unsigned my_index = next_ir_index++;

   _mesa_hash_table_insert(index_map, ir, (void *)(uintptr_t) my_index);

   const char *mode_str;
   switch (ir->data.mode) {
   case ir_var_auto: mode_str = "ir_var_auto"; break;
   case ir_var_uniform: mode_str = "ir_var_uniform"; break;
   case ir_var_shader_storage: mode_str = "ir_var_shader_storage"; break;
   case ir_var_shader_shared: mode_str = "ir_var_shader_shared"; break;
   case ir_var_shader_in: mode_str = "ir_var_shader_in"; break;
   case ir_var_shader_out: mode_str = "ir_var_shader_out"; break;
   case ir_var_function_in: mode_str = "ir_var_function_in"; break;
   case ir_var_function_out: mode_str = "ir_var_function_out"; break;
   case ir_var_function_inout: mode_str = "ir_var_function_inout"; break;
   case ir_var_const_in: mode_str = "ir_var_const_in"; break;
   case ir_var_system_value: mode_str = "ir_var_system_value"; break;
   case ir_var_temporary: mode_str = "ir_var_temporary"; break;
   default:
      unreachable("Invalid variable mode");
   }

   if (ir->data.mode == ir_var_temporary) {
      print_with_indent("ir_variable *const r%04X = body.make_temp(glsl_type::%s_type, \"%s\");\n",
                        my_index,
                        ir->type->name,
                        ir->name);
   } else {
      print_with_indent("ir_variable *const r%04X = new(mem_ctx) ir_variable(glsl_type::%s_type, \"%s\", %s);\n",
                        my_index,
                        ir->type->name,
                        ir->name,
                        mode_str);

      switch (ir->data.mode) {
      case ir_var_function_in:
      case ir_var_function_out:
      case ir_var_function_inout:
      case ir_var_const_in:
         print_with_indent("sig_parameters.push_tail(r%04X);\n", my_index);
         break;
      default:
         print_with_indent("body.emit(r%04X);\n", my_index);
         break;
      }
   }

   return visit_continue;
}

void
ir_builder_print_visitor::print_without_declaration(const ir_dereference_variable *ir)
{
   const struct hash_entry *const he =
      _mesa_hash_table_search(index_map, ir->var);

   print_without_indent("r%04X", (unsigned)(uintptr_t) he->data);
}

ir_visitor_status
ir_builder_print_visitor::visit(ir_dereference_variable *ir)
{
   const struct hash_entry *const he =
      _mesa_hash_table_search(index_map, ir->var);

   if (he != NULL)
      _mesa_hash_table_insert(index_map, ir, he->data);

   return visit_continue;
}

ir_visitor_status
ir_builder_print_visitor::visit_enter(ir_function_signature *ir)
{
   if (!ir->is_defined)
      return visit_continue_with_parent;

   print_with_indent("ir_function_signature *\n"
                     "%s(void *mem_ctx, builtin_available_predicate avail)\n"
                     "{\n",
                     ir->function_name());
   indentation++;
   print_with_indent("ir_function_signature *const sig =\n");
   print_with_indent("   new(mem_ctx) ir_function_signature(glsl_type::%s_type, avail);\n",
                     ir->return_type->name);

   print_with_indent("ir_factory body(&sig->body, mem_ctx);\n");
   print_with_indent("sig->is_defined = true;\n\n");

   if (!ir->parameters.is_empty())
      print_with_indent("exec_list sig_parameters;\n\n");

   return visit_continue;
}

ir_visitor_status
ir_builder_print_visitor::visit_leave(ir_function_signature *ir)
{
   if (!ir->parameters.is_empty())
      print_with_indent("sig->replace_parameters(&sig_parameters);\n");

   print_with_indent("return sig;\n");
   indentation--;
   print_with_indent("}\n");
   return visit_continue;
}

void
ir_builder_print_visitor::print_without_declaration(const ir_constant *ir)
{
  if (ir->type->is_scalar()) {
      switch (ir->type->base_type) {
      case GLSL_TYPE_UINT:
         print_without_indent("body.constant(%uu)", ir->value.u[0]);
         return;
      case GLSL_TYPE_INT:
         print_without_indent("body.constant(int(%d))", ir->value.i[0]);
         return;
      case GLSL_TYPE_FLOAT:
         print_without_indent("body.constant(%ff)", ir->value.f[0]);
         return;
      case GLSL_TYPE_BOOL:
         print_without_indent("body.constant(%s)",
                              ir->value.i[0] != 0 ? "true" : "false");
         return;
      default:
         break;
      }
   }

   ir_constant_data all_zero;
   memset(&all_zero, 0, sizeof(all_zero));

   if (memcmp(&ir->value, &all_zero, sizeof(all_zero)) == 0) {
      print_without_indent("ir_constant::zero(mem_ctx, glsl_type::%s_type)",
                           ir->type->name);
   }
}

ir_visitor_status
ir_builder_print_visitor::visit(ir_constant *ir)
{
   const unsigned my_index = next_ir_index++;

   _mesa_hash_table_insert(index_map, ir, (void *)(uintptr_t) my_index);

   if (ir->type == glsl_type::uint_type ||
       ir->type == glsl_type::int_type ||
       ir->type == glsl_type::float_type ||
       ir->type == glsl_type::bool_type) {
      print_with_indent("ir_constant *const r%04X = ", my_index);
      print_without_declaration(ir);
      print_without_indent(";\n");
      return visit_continue;
   }

   ir_constant_data all_zero;
   memset(&all_zero, 0, sizeof(all_zero));

   if (memcmp(&ir->value, &all_zero, sizeof(all_zero)) == 0) {
      print_with_indent("ir_constant *const r%04X = ", my_index);
      print_without_declaration(ir);
      print_without_indent(";\n");
   } else {
      print_with_indent("ir_constant_data r%04X_data;\n", my_index);
      print_with_indent("memset(&r%04X_data, 0, sizeof(ir_constant_data));\n",
                        my_index);
      for (unsigned i = 0; i < 16; i++) {
         switch (ir->type->base_type) {
         case GLSL_TYPE_UINT:
            if (ir->value.u[i] != 0)
               print_with_indent("r%04X_data.u[%u] = %u;\n",
                                    my_index, i, ir->value.u[i]);
            break;
         case GLSL_TYPE_INT:
            if (ir->value.i[i] != 0)
               print_with_indent("r%04X_data.i[%u] = %i;\n",
                                    my_index, i, ir->value.i[i]);
            break;
         case GLSL_TYPE_FLOAT:
            if (ir->value.u[i] != 0)
               print_with_indent("r%04X_data.u[%u] = 0x%08x; /* %f */\n",
                                    my_index,
                                    i,
                                    ir->value.u[i],
                                    ir->value.f[i]);
            break;
         case GLSL_TYPE_DOUBLE: {
            uint64_t v;

            STATIC_ASSERT(sizeof(double) == sizeof(uint64_t));

            memcpy(&v, &ir->value.d[i], sizeof(v));
            if (v != 0)
               print_with_indent("r%04X_data.u64[%u] = 0x%016" PRIx64 "; /* %g */\n",
                                    my_index, i, v, ir->value.d[i]);
            break;
         }
         case GLSL_TYPE_UINT64:
            if (ir->value.u64[i] != 0)
               print_with_indent("r%04X_data.u64[%u] = %" PRIu64 ";\n",
                                    my_index,
                                    i,
                                    ir->value.u64[i]);
            break;
         case GLSL_TYPE_INT64:
            if (ir->value.i64[i] != 0)
               print_with_indent("r%04X_data.i64[%u] = %" PRId64 ";\n",
                                    my_index,
                                    i,
                                    ir->value.i64[i]);
            break;
         case GLSL_TYPE_BOOL:
            if (ir->value.u[i] != 0)
               print_with_indent("r%04X_data.u[%u] = 1;\n", my_index, i);
            break;
         default:
            unreachable("Invalid constant type");
         }
      }

      print_with_indent("ir_constant *const r%04X = new(mem_ctx) ir_constant(glsl_type::%s_type, &r%04X_data);\n",
                        my_index,
                        ir->type->name,
                        my_index);
   }

   return visit_continue;
}

void
ir_builder_print_visitor::print_without_declaration(const ir_swizzle *ir)
{
   const struct hash_entry *const he =
      _mesa_hash_table_search(index_map, ir->val);

   if (ir->mask.num_components == 1) {
      static const char swiz[4] = { 'x', 'y', 'z', 'w' };

      if (is_simple_operand(ir->val)) {
         print_without_indent("swizzle_%c(", swiz[ir->mask.x]);
         print_without_declaration(ir->val);
         print_without_indent(")");
      } else {
         assert(he);
         print_without_indent("swizzle_%c(r%04X)",
                              swiz[ir->mask.x],
                              (unsigned)(uintptr_t) he->data);
      }
   } else {
      static const char swiz[4] = { 'X', 'Y', 'Z', 'W' };

      assert(he);
      print_without_indent("swizzle(r%04X, MAKE_SWIZZLE4(SWIZZLE_%c, SWIZZLE_%c, SWIZZLE_%c, SWIZZLE_%c), %u)",
                           (unsigned)(uintptr_t) he->data,
                           swiz[ir->mask.x],
                           swiz[ir->mask.y],
                           swiz[ir->mask.z],
                           swiz[ir->mask.w],
                           ir->mask.num_components);
   }
}

ir_visitor_status
ir_builder_print_visitor::visit_leave(ir_swizzle *ir)
{
   const unsigned my_index = next_ir_index++;

   _mesa_hash_table_insert(index_map, ir, (void *)(uintptr_t) my_index);

   print_with_indent("ir_swizzle *const r%04X = ", my_index);
   print_without_declaration(ir);
   print_without_indent(";\n");

   return visit_continue;
}

ir_visitor_status
ir_builder_print_visitor::visit_enter(ir_assignment *ir)
{
   ir_expression *const rhs_expr = ir->rhs->as_expression();

   if (!is_simple_operand(ir->rhs) && rhs_expr == NULL)
      return visit_continue;

   if (rhs_expr != NULL) {
      const unsigned num_op = rhs_expr->num_operands;

      for (unsigned i = 0; i < num_op; i++) {
         if (is_simple_operand(rhs_expr->operands[i]))
            continue;

         rhs_expr->operands[i]->accept(this);
      }
   }

   ir_visitor_status s;

   this->in_assignee = true;
   s = ir->lhs->accept(this);
   this->in_assignee = false;
   if (s != visit_continue)
      return (s == visit_continue_with_parent) ? visit_continue : s;

   assert(ir->condition == NULL);

   const struct hash_entry *const he_lhs =
      _mesa_hash_table_search(index_map, ir->lhs);

   print_with_indent("body.emit(assign(r%04X, ",
                     (unsigned)(uintptr_t) he_lhs->data);
   print_without_declaration(ir->rhs);
   print_without_indent(", 0x%02x));\n\n", ir->write_mask);

   return visit_continue_with_parent;
}

ir_visitor_status
ir_builder_print_visitor::visit_leave(ir_assignment *ir)
{
   const struct hash_entry *const he_lhs =
      _mesa_hash_table_search(index_map, ir->lhs);

   const struct hash_entry *const he_rhs =
      _mesa_hash_table_search(index_map, ir->rhs);

   assert(ir->condition == NULL);
   assert(ir->lhs && ir->rhs);

   print_with_indent("body.emit(assign(r%04X, r%04X, 0x%02x));\n\n",
                     (unsigned)(uintptr_t) he_lhs->data,
                     (unsigned)(uintptr_t) he_rhs->data,
                     ir->write_mask);

   return visit_continue;
}

void
ir_builder_print_visitor::print_without_declaration(const ir_expression *ir)
{
   const unsigned num_op = ir->num_operands;

   static const char *const arity[] = {
      "", "unop", "binop", "triop", "quadop"
   };

   switch (ir->operation) {
   case ir_unop_neg:
   case ir_binop_add:
   case ir_binop_sub:
   case ir_binop_mul:
   case ir_binop_imul_high:
   case ir_binop_less:
   case ir_binop_gequal:
   case ir_binop_equal:
   case ir_binop_nequal:
   case ir_binop_lshift:
   case ir_binop_rshift:
   case ir_binop_bit_and:
   case ir_binop_bit_xor:
   case ir_binop_bit_or:
   case ir_binop_logic_and:
   case ir_binop_logic_xor:
   case ir_binop_logic_or:
      print_without_indent("%s(",
                           ir_expression_operation_enum_strings[ir->operation]);
      break;
   default:
      print_without_indent("expr(ir_%s_%s, ",
                           arity[num_op],
                           ir_expression_operation_enum_strings[ir->operation]);
      break;
   }

   for (unsigned i = 0; i < num_op; i++) {
      if (is_simple_operand(ir->operands[i]))
         print_without_declaration(ir->operands[i]);
      else {
         const struct hash_entry *const he =
            _mesa_hash_table_search(index_map, ir->operands[i]);

         print_without_indent("r%04X", (unsigned)(uintptr_t) he->data);
      }

      if (i < num_op - 1)
         print_without_indent(", ");
   }

   print_without_indent(")");
}

ir_visitor_status
ir_builder_print_visitor::visit_enter(ir_expression *ir)
{
   const unsigned num_op = ir->num_operands;

   for (unsigned i = 0; i < num_op; i++) {
      if (is_simple_operand(ir->operands[i]))
         continue;

      ir->operands[i]->accept(this);
   }

   const unsigned my_index = next_ir_index++;

   _mesa_hash_table_insert(index_map, ir, (void *)(uintptr_t) my_index);

   print_with_indent("ir_expression *const r%04X = ", my_index);
   print_without_declaration(ir);
   print_without_indent(";\n");

   return visit_continue_with_parent;
}

ir_visitor_status
ir_builder_print_visitor::visit_enter(ir_if *ir)
{
   const unsigned my_index = next_ir_index++;

   print_with_indent("/* IF CONDITION */\n");

   ir_visitor_status s = ir->condition->accept(this);
   if (s != visit_continue)
      return (s == visit_continue_with_parent) ? visit_continue : s;

   const struct hash_entry *const he =
      _mesa_hash_table_search(index_map, ir->condition);

   print_with_indent("ir_if *f%04X = new(mem_ctx) ir_if(operand(r%04X).val);\n",
                     my_index,
                     (unsigned)(uintptr_t) he->data);
   print_with_indent("exec_list *const f%04X_parent_instructions = body.instructions;\n\n",
                     my_index);

   indentation++;
   print_with_indent("/* THEN INSTRUCTIONS */\n");
   print_with_indent("body.instructions = &f%04X->then_instructions;\n\n",
                     my_index);

   if (s != visit_continue_with_parent) {
      s = visit_list_elements(this, &ir->then_instructions);
      if (s == visit_stop)
        return s;
   }

   print_without_indent("\n");

   if (!ir->else_instructions.is_empty()) {
      print_with_indent("/* ELSE INSTRUCTIONS */\n");
      print_with_indent("body.instructions = &f%04X->else_instructions;\n\n",
              my_index);

      if (s != visit_continue_with_parent) {
         s = visit_list_elements(this, &ir->else_instructions);
         if (s == visit_stop)
            return s;
      }

      print_without_indent("\n");
   }

   indentation--;

   print_with_indent("body.instructions = f%04X_parent_instructions;\n",
                     my_index);
   print_with_indent("body.emit(f%04X);\n\n",
                     my_index);
   print_with_indent("/* END IF */\n\n");

   return visit_continue_with_parent;
}

ir_visitor_status
ir_builder_print_visitor::visit_leave(ir_return *ir)
{
   const struct hash_entry *const he =
      _mesa_hash_table_search(index_map, ir->value);

   print_with_indent("body.emit(ret(r%04X));\n\n",
                     (unsigned)(uintptr_t) he->data);

   return visit_continue;
}

ir_visitor_status
ir_builder_print_visitor::visit_enter(ir_texture *ir)
{
   print_with_indent("\nUnsupported IR is encountered: texture functions are not supported. Exiting.\n");

   return visit_stop;
}

ir_visitor_status
ir_builder_print_visitor::visit_leave(ir_call *ir)
{
   const unsigned my_index = next_ir_index++;

   print_without_indent("\n");
   print_with_indent("/* CALL %s */\n", ir->callee_name());
   print_with_indent("exec_list r%04X_parameters;\n", my_index);

   foreach_in_list(ir_dereference_variable, param, &ir->actual_parameters) {
      const struct hash_entry *const he =
         _mesa_hash_table_search(index_map, param);

      print_with_indent("r%04X_parameters.push_tail(operand(r%04X).val);\n",
                        my_index,
                        (unsigned)(uintptr_t) he->data);
   }

   char return_deref_string[32];
   if (ir->return_deref) {
      const struct hash_entry *const he =
         _mesa_hash_table_search(index_map, ir->return_deref);

      snprintf(return_deref_string, sizeof(return_deref_string),
               "operand(r%04X).val", (unsigned)(uintptr_t) he->data);
   } else {
      strcpy(return_deref_string, "NULL");
   }

   print_with_indent("body.emit(new(mem_ctx) ir_call(shader->symbols->get_function(\"%s\"),\n",
                     ir->callee_name());
   print_with_indent("                               %s, &r%04X_parameters);\n\n",
                     return_deref_string,
                     my_index);
   return visit_continue;
}

ir_visitor_status
ir_builder_print_visitor::visit_enter(ir_loop *ir)
{
   const unsigned my_index = next_ir_index++;

   _mesa_hash_table_insert(index_map, ir, (void *)(uintptr_t) my_index);

   print_with_indent("/* LOOP BEGIN */\n");
   print_with_indent("ir_loop *f%04X = new(mem_ctx) ir_loop();\n", my_index);
   print_with_indent("exec_list *const f%04X_parent_instructions = body.instructions;\n\n",
                     my_index);

   indentation++;

   print_with_indent("body.instructions = &f%04X->body_instructions;\n\n",
                     my_index);

   return visit_continue;
}

ir_visitor_status
ir_builder_print_visitor::visit_leave(ir_loop *ir)
{
   const struct hash_entry *const he =
      _mesa_hash_table_search(index_map, ir);

   indentation--;

   print_with_indent("/* LOOP END */\n\n");
   print_with_indent("body.instructions = f%04X_parent_instructions;\n",
                     (unsigned)(uintptr_t) he->data);
   print_with_indent("body.emit(f%04X);\n\n",
                     (unsigned)(uintptr_t) he->data);

   return visit_continue;
}

ir_visitor_status
ir_builder_print_visitor::visit(ir_loop_jump *ir)
{
   print_with_indent("body.emit(new(mem_ctx) ir_loop_jump(ir_loop_jump::jump_%s));\n\n",
                     ir->is_break() ? "break" : "continue");
   return visit_continue;
}
