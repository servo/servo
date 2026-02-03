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

#include "ir_reader.h"
#include "glsl_parser_extras.h"
#include "compiler/glsl_types.h"
#include "s_expression.h"

static const bool debug = false;

namespace {

class ir_reader {
public:
   ir_reader(_mesa_glsl_parse_state *);

   void read(exec_list *instructions, const char *src, bool scan_for_protos);

private:
   void *mem_ctx;
   _mesa_glsl_parse_state *state;

   void ir_read_error(s_expression *, const char *fmt, ...);

   const glsl_type *read_type(s_expression *);

   void scan_for_prototypes(exec_list *, s_expression *);
   ir_function *read_function(s_expression *, bool skip_body);
   void read_function_sig(ir_function *, s_expression *, bool skip_body);

   void read_instructions(exec_list *, s_expression *, ir_loop *);
   ir_instruction *read_instruction(s_expression *, ir_loop *);
   ir_variable *read_declaration(s_expression *);
   ir_if *read_if(s_expression *, ir_loop *);
   ir_loop *read_loop(s_expression *);
   ir_call *read_call(s_expression *);
   ir_return *read_return(s_expression *);
   ir_rvalue *read_rvalue(s_expression *);
   ir_assignment *read_assignment(s_expression *);
   ir_expression *read_expression(s_expression *);
   ir_swizzle *read_swizzle(s_expression *);
   ir_constant *read_constant(s_expression *);
   ir_texture *read_texture(s_expression *);
   ir_emit_vertex *read_emit_vertex(s_expression *);
   ir_end_primitive *read_end_primitive(s_expression *);
   ir_barrier *read_barrier(s_expression *);

   ir_dereference *read_dereference(s_expression *);
   ir_dereference_variable *read_var_ref(s_expression *);
};

} /* anonymous namespace */

ir_reader::ir_reader(_mesa_glsl_parse_state *state) : state(state)
{
   this->mem_ctx = state;
}

void
_mesa_glsl_read_ir(_mesa_glsl_parse_state *state, exec_list *instructions,
		   const char *src, bool scan_for_protos)
{
   ir_reader r(state);
   r.read(instructions, src, scan_for_protos);
}

void
ir_reader::read(exec_list *instructions, const char *src, bool scan_for_protos)
{
   void *sx_mem_ctx = ralloc_context(NULL);
   s_expression *expr = s_expression::read_expression(sx_mem_ctx, src);
   if (expr == NULL) {
      ir_read_error(NULL, "couldn't parse S-Expression.");
      return;
   }

   if (scan_for_protos) {
      scan_for_prototypes(instructions, expr);
      if (state->error)
	 return;
   }

   read_instructions(instructions, expr, NULL);
   ralloc_free(sx_mem_ctx);

   if (debug)
      validate_ir_tree(instructions);
}

void
ir_reader::ir_read_error(s_expression *expr, const char *fmt, ...)
{
   va_list ap;

   state->error = true;

   if (state->current_function != NULL)
      ralloc_asprintf_append(&state->info_log, "In function %s:\n",
			     state->current_function->function_name());
   ralloc_strcat(&state->info_log, "error: ");

   va_start(ap, fmt);
   ralloc_vasprintf_append(&state->info_log, fmt, ap);
   va_end(ap);
   ralloc_strcat(&state->info_log, "\n");

   if (expr != NULL) {
      ralloc_strcat(&state->info_log, "...in this context:\n   ");
      expr->print();
      ralloc_strcat(&state->info_log, "\n\n");
   }
}

const glsl_type *
ir_reader::read_type(s_expression *expr)
{
   s_expression *s_base_type;
   s_int *s_size;

   s_pattern pat[] = { "array", s_base_type, s_size };
   if (MATCH(expr, pat)) {
      const glsl_type *base_type = read_type(s_base_type);
      if (base_type == NULL) {
	 ir_read_error(NULL, "when reading base type of array type");
	 return NULL;
      }

      return glsl_type::get_array_instance(base_type, s_size->value());
   }

   s_symbol *type_sym = SX_AS_SYMBOL(expr);
   if (type_sym == NULL) {
      ir_read_error(expr, "expected <type>");
      return NULL;
   }

   const glsl_type *type = state->symbols->get_type(type_sym->value());
   if (type == NULL)
      ir_read_error(expr, "invalid type: %s", type_sym->value());

   return type;
}


void
ir_reader::scan_for_prototypes(exec_list *instructions, s_expression *expr)
{
   s_list *list = SX_AS_LIST(expr);
   if (list == NULL) {
      ir_read_error(expr, "Expected (<instruction> ...); found an atom.");
      return;
   }

   foreach_in_list(s_list, sub, &list->subexpressions) {
      if (!sub->is_list())
	 continue; // not a (function ...); ignore it.

      s_symbol *tag = SX_AS_SYMBOL(sub->subexpressions.get_head());
      if (tag == NULL || strcmp(tag->value(), "function") != 0)
	 continue; // not a (function ...); ignore it.

      ir_function *f = read_function(sub, true);
      if (f == NULL)
	 return;
      instructions->push_tail(f);
   }
}

ir_function *
ir_reader::read_function(s_expression *expr, bool skip_body)
{
   bool added = false;
   s_symbol *name;

   s_pattern pat[] = { "function", name };
   if (!PARTIAL_MATCH(expr, pat)) {
      ir_read_error(expr, "Expected (function <name> (signature ...) ...)");
      return NULL;
   }

   ir_function *f = state->symbols->get_function(name->value());
   if (f == NULL) {
      f = new(mem_ctx) ir_function(name->value());
      added = state->symbols->add_function(f);
      assert(added);
   }

   /* Skip over "function" tag and function name (which are guaranteed to be
    * present by the above PARTIAL_MATCH call).
    */
   exec_node *node = ((s_list *) expr)->subexpressions.get_head_raw()->next->next;
   for (/* nothing */; !node->is_tail_sentinel(); node = node->next) {
      s_expression *s_sig = (s_expression *) node;
      read_function_sig(f, s_sig, skip_body);
   }
   return added ? f : NULL;
}

static bool
always_available(const _mesa_glsl_parse_state *)
{
   return true;
}

void
ir_reader::read_function_sig(ir_function *f, s_expression *expr, bool skip_body)
{
   s_expression *type_expr;
   s_list *paramlist;
   s_list *body_list;

   s_pattern pat[] = { "signature", type_expr, paramlist, body_list };
   if (!MATCH(expr, pat)) {
      ir_read_error(expr, "Expected (signature <type> (parameters ...) "
			  "(<instruction> ...))");
      return;
   }

   const glsl_type *return_type = read_type(type_expr);
   if (return_type == NULL)
      return;

   s_symbol *paramtag = SX_AS_SYMBOL(paramlist->subexpressions.get_head());
   if (paramtag == NULL || strcmp(paramtag->value(), "parameters") != 0) {
      ir_read_error(paramlist, "Expected (parameters ...)");
      return;
   }

   // Read the parameters list into a temporary place.
   exec_list hir_parameters;
   state->symbols->push_scope();

   /* Skip over the "parameters" tag. */
   exec_node *node = paramlist->subexpressions.get_head_raw()->next;
   for (/* nothing */; !node->is_tail_sentinel(); node = node->next) {
      ir_variable *var = read_declaration((s_expression *) node);
      if (var == NULL)
	 return;

      hir_parameters.push_tail(var);
   }

   ir_function_signature *sig =
      f->exact_matching_signature(state, &hir_parameters);
   if (sig == NULL && skip_body) {
      /* If scanning for prototypes, generate a new signature. */
      /* ir_reader doesn't know what languages support a given built-in, so
       * just say that they're always available.  For now, other mechanisms
       * guarantee the right built-ins are available.
       */
      sig = new(mem_ctx) ir_function_signature(return_type, always_available);
      f->add_signature(sig);
   } else if (sig != NULL) {
      const char *badvar = sig->qualifiers_match(&hir_parameters);
      if (badvar != NULL) {
	 ir_read_error(expr, "function `%s' parameter `%s' qualifiers "
		       "don't match prototype", f->name, badvar);
	 return;
      }

      if (sig->return_type != return_type) {
	 ir_read_error(expr, "function `%s' return type doesn't "
		       "match prototype", f->name);
	 return;
      }
   } else {
      /* No prototype for this body exists - skip it. */
      state->symbols->pop_scope();
      return;
   }
   assert(sig != NULL);

   sig->replace_parameters(&hir_parameters);

   if (!skip_body && !body_list->subexpressions.is_empty()) {
      if (sig->is_defined) {
	 ir_read_error(expr, "function %s redefined", f->name);
	 return;
      }
      state->current_function = sig;
      read_instructions(&sig->body, body_list, NULL);
      state->current_function = NULL;
      sig->is_defined = true;
   }

   state->symbols->pop_scope();
}

void
ir_reader::read_instructions(exec_list *instructions, s_expression *expr,
			     ir_loop *loop_ctx)
{
   // Read in a list of instructions
   s_list *list = SX_AS_LIST(expr);
   if (list == NULL) {
      ir_read_error(expr, "Expected (<instruction> ...); found an atom.");
      return;
   }

   foreach_in_list(s_expression, sub, &list->subexpressions) {
      ir_instruction *ir = read_instruction(sub, loop_ctx);
      if (ir != NULL) {
	 /* Global variable declarations should be moved to the top, before
	  * any functions that might use them.  Functions are added to the
	  * instruction stream when scanning for prototypes, so without this
	  * hack, they always appear before variable declarations.
	  */
	 if (state->current_function == NULL && ir->as_variable() != NULL)
	    instructions->push_head(ir);
	 else
	    instructions->push_tail(ir);
      }
   }
}


ir_instruction *
ir_reader::read_instruction(s_expression *expr, ir_loop *loop_ctx)
{
   s_symbol *symbol = SX_AS_SYMBOL(expr);
   if (symbol != NULL) {
      if (strcmp(symbol->value(), "break") == 0 && loop_ctx != NULL)
	 return new(mem_ctx) ir_loop_jump(ir_loop_jump::jump_break);
      if (strcmp(symbol->value(), "continue") == 0 && loop_ctx != NULL)
	 return new(mem_ctx) ir_loop_jump(ir_loop_jump::jump_continue);
   }

   s_list *list = SX_AS_LIST(expr);
   if (list == NULL || list->subexpressions.is_empty()) {
      ir_read_error(expr, "Invalid instruction.\n");
      return NULL;
   }

   s_symbol *tag = SX_AS_SYMBOL(list->subexpressions.get_head());
   if (tag == NULL) {
      ir_read_error(expr, "expected instruction tag");
      return NULL;
   }

   ir_instruction *inst = NULL;
   if (strcmp(tag->value(), "declare") == 0) {
      inst = read_declaration(list);
   } else if (strcmp(tag->value(), "assign") == 0) {
      inst = read_assignment(list);
   } else if (strcmp(tag->value(), "if") == 0) {
      inst = read_if(list, loop_ctx);
   } else if (strcmp(tag->value(), "loop") == 0) {
      inst = read_loop(list);
   } else if (strcmp(tag->value(), "call") == 0) {
      inst = read_call(list);
   } else if (strcmp(tag->value(), "return") == 0) {
      inst = read_return(list);
   } else if (strcmp(tag->value(), "function") == 0) {
      inst = read_function(list, false);
   } else if (strcmp(tag->value(), "emit-vertex") == 0) {
      inst = read_emit_vertex(list);
   } else if (strcmp(tag->value(), "end-primitive") == 0) {
      inst = read_end_primitive(list);
   } else if (strcmp(tag->value(), "barrier") == 0) {
      inst = read_barrier(list);
   } else {
      inst = read_rvalue(list);
      if (inst == NULL)
	 ir_read_error(NULL, "when reading instruction");
   }
   return inst;
}

ir_variable *
ir_reader::read_declaration(s_expression *expr)
{
   s_list *s_quals;
   s_expression *s_type;
   s_symbol *s_name;

   s_pattern pat[] = { "declare", s_quals, s_type, s_name };
   if (!MATCH(expr, pat)) {
      ir_read_error(expr, "expected (declare (<qualifiers>) <type> <name>)");
      return NULL;
   }

   const glsl_type *type = read_type(s_type);
   if (type == NULL)
      return NULL;

   ir_variable *var = new(mem_ctx) ir_variable(type, s_name->value(),
					       ir_var_auto);

   foreach_in_list(s_symbol, qualifier, &s_quals->subexpressions) {
      if (!qualifier->is_symbol()) {
	 ir_read_error(expr, "qualifier list must contain only symbols");
	 return NULL;
      }

      // FINISHME: Check for duplicate/conflicting qualifiers.
      if (strcmp(qualifier->value(), "centroid") == 0) {
	 var->data.centroid = 1;
      } else if (strcmp(qualifier->value(), "sample") == 0) {
         var->data.sample = 1;
      } else if (strcmp(qualifier->value(), "patch") == 0) {
         var->data.patch = 1;
      } else if (strcmp(qualifier->value(), "explicit_invariant") == 0) {
         var->data.explicit_invariant = true;
      } else if (strcmp(qualifier->value(), "invariant") == 0) {
         var->data.invariant = true;
      } else if (strcmp(qualifier->value(), "uniform") == 0) {
	 var->data.mode = ir_var_uniform;
      } else if (strcmp(qualifier->value(), "shader_storage") == 0) {
	 var->data.mode = ir_var_shader_storage;
      } else if (strcmp(qualifier->value(), "auto") == 0) {
	 var->data.mode = ir_var_auto;
      } else if (strcmp(qualifier->value(), "in") == 0) {
	 var->data.mode = ir_var_function_in;
      } else if (strcmp(qualifier->value(), "shader_in") == 0) {
         var->data.mode = ir_var_shader_in;
      } else if (strcmp(qualifier->value(), "const_in") == 0) {
	 var->data.mode = ir_var_const_in;
      } else if (strcmp(qualifier->value(), "out") == 0) {
	 var->data.mode = ir_var_function_out;
      } else if (strcmp(qualifier->value(), "shader_out") == 0) {
	 var->data.mode = ir_var_shader_out;
      } else if (strcmp(qualifier->value(), "inout") == 0) {
	 var->data.mode = ir_var_function_inout;
      } else if (strcmp(qualifier->value(), "temporary") == 0) {
	 var->data.mode = ir_var_temporary;
      } else if (strcmp(qualifier->value(), "stream1") == 0) {
	 var->data.stream = 1;
      } else if (strcmp(qualifier->value(), "stream2") == 0) {
	 var->data.stream = 2;
      } else if (strcmp(qualifier->value(), "stream3") == 0) {
	 var->data.stream = 3;
      } else if (strcmp(qualifier->value(), "smooth") == 0) {
	 var->data.interpolation = INTERP_MODE_SMOOTH;
      } else if (strcmp(qualifier->value(), "flat") == 0) {
	 var->data.interpolation = INTERP_MODE_FLAT;
      } else if (strcmp(qualifier->value(), "noperspective") == 0) {
	 var->data.interpolation = INTERP_MODE_NOPERSPECTIVE;
      } else {
	 ir_read_error(expr, "unknown qualifier: %s", qualifier->value());
	 return NULL;
      }
   }

   // Add the variable to the symbol table
   state->symbols->add_variable(var);

   return var;
}


ir_if *
ir_reader::read_if(s_expression *expr, ir_loop *loop_ctx)
{
   s_expression *s_cond;
   s_expression *s_then;
   s_expression *s_else;

   s_pattern pat[] = { "if", s_cond, s_then, s_else };
   if (!MATCH(expr, pat)) {
      ir_read_error(expr, "expected (if <condition> (<then>...) (<else>...))");
      return NULL;
   }

   ir_rvalue *condition = read_rvalue(s_cond);
   if (condition == NULL) {
      ir_read_error(NULL, "when reading condition of (if ...)");
      return NULL;
   }

   ir_if *iff = new(mem_ctx) ir_if(condition);

   read_instructions(&iff->then_instructions, s_then, loop_ctx);
   read_instructions(&iff->else_instructions, s_else, loop_ctx);
   if (state->error) {
      delete iff;
      iff = NULL;
   }
   return iff;
}


ir_loop *
ir_reader::read_loop(s_expression *expr)
{
   s_expression *s_body;

   s_pattern loop_pat[] = { "loop", s_body };
   if (!MATCH(expr, loop_pat)) {
      ir_read_error(expr, "expected (loop <body>)");
      return NULL;
   }

   ir_loop *loop = new(mem_ctx) ir_loop;

   read_instructions(&loop->body_instructions, s_body, loop);
   if (state->error) {
      delete loop;
      loop = NULL;
   }
   return loop;
}


ir_return *
ir_reader::read_return(s_expression *expr)
{
   s_expression *s_retval;

   s_pattern return_value_pat[] = { "return", s_retval};
   s_pattern return_void_pat[] = { "return" };
   if (MATCH(expr, return_value_pat)) {
      ir_rvalue *retval = read_rvalue(s_retval);
      if (retval == NULL) {
         ir_read_error(NULL, "when reading return value");
         return NULL;
      }
      return new(mem_ctx) ir_return(retval);
   } else if (MATCH(expr, return_void_pat)) {
      return new(mem_ctx) ir_return;
   } else {
      ir_read_error(expr, "expected (return <rvalue>) or (return)");
      return NULL;
   }
}


ir_rvalue *
ir_reader::read_rvalue(s_expression *expr)
{
   s_list *list = SX_AS_LIST(expr);
   if (list == NULL || list->subexpressions.is_empty())
      return NULL;

   s_symbol *tag = SX_AS_SYMBOL(list->subexpressions.get_head());
   if (tag == NULL) {
      ir_read_error(expr, "expected rvalue tag");
      return NULL;
   }

   ir_rvalue *rvalue = read_dereference(list);
   if (rvalue != NULL || state->error)
      return rvalue;
   else if (strcmp(tag->value(), "swiz") == 0) {
      rvalue = read_swizzle(list);
   } else if (strcmp(tag->value(), "expression") == 0) {
      rvalue = read_expression(list);
   } else if (strcmp(tag->value(), "constant") == 0) {
      rvalue = read_constant(list);
   } else {
      rvalue = read_texture(list);
      if (rvalue == NULL && !state->error)
	 ir_read_error(expr, "unrecognized rvalue tag: %s", tag->value());
   }

   return rvalue;
}

ir_assignment *
ir_reader::read_assignment(s_expression *expr)
{
   s_expression *cond_expr = NULL;
   s_expression *lhs_expr, *rhs_expr;
   s_list       *mask_list;

   s_pattern pat4[] = { "assign",            mask_list, lhs_expr, rhs_expr };
   s_pattern pat5[] = { "assign", cond_expr, mask_list, lhs_expr, rhs_expr };
   if (!MATCH(expr, pat4) && !MATCH(expr, pat5)) {
      ir_read_error(expr, "expected (assign [<condition>] (<write mask>) "
			  "<lhs> <rhs>)");
      return NULL;
   }

   ir_rvalue *condition = NULL;
   if (cond_expr != NULL) {
      condition = read_rvalue(cond_expr);
      if (condition == NULL) {
	 ir_read_error(NULL, "when reading condition of assignment");
	 return NULL;
      }
   }

   unsigned mask = 0;

   s_symbol *mask_symbol;
   s_pattern mask_pat[] = { mask_symbol };
   if (MATCH(mask_list, mask_pat)) {
      const char *mask_str = mask_symbol->value();
      unsigned mask_length = strlen(mask_str);
      if (mask_length > 4) {
	 ir_read_error(expr, "invalid write mask: %s", mask_str);
	 return NULL;
      }

      const unsigned idx_map[] = { 3, 0, 1, 2 }; /* w=bit 3, x=0, y=1, z=2 */

      for (unsigned i = 0; i < mask_length; i++) {
	 if (mask_str[i] < 'w' || mask_str[i] > 'z') {
	    ir_read_error(expr, "write mask contains invalid character: %c",
			  mask_str[i]);
	    return NULL;
	 }
	 mask |= 1 << idx_map[mask_str[i] - 'w'];
      }
   } else if (!mask_list->subexpressions.is_empty()) {
      ir_read_error(mask_list, "expected () or (<write mask>)");
      return NULL;
   }

   ir_dereference *lhs = read_dereference(lhs_expr);
   if (lhs == NULL) {
      ir_read_error(NULL, "when reading left-hand side of assignment");
      return NULL;
   }

   ir_rvalue *rhs = read_rvalue(rhs_expr);
   if (rhs == NULL) {
      ir_read_error(NULL, "when reading right-hand side of assignment");
      return NULL;
   }

   if (mask == 0 && (lhs->type->is_vector() || lhs->type->is_scalar())) {
      ir_read_error(expr, "non-zero write mask required.");
      return NULL;
   }

   return new(mem_ctx) ir_assignment(lhs, rhs, condition, mask);
}

ir_call *
ir_reader::read_call(s_expression *expr)
{
   s_symbol *name;
   s_list *params;
   s_list *s_return = NULL;

   ir_dereference_variable *return_deref = NULL;

   s_pattern void_pat[] = { "call", name, params };
   s_pattern non_void_pat[] = { "call", name, s_return, params };
   if (MATCH(expr, non_void_pat)) {
      return_deref = read_var_ref(s_return);
      if (return_deref == NULL) {
	 ir_read_error(s_return, "when reading a call's return storage");
	 return NULL;
      }
   } else if (!MATCH(expr, void_pat)) {
      ir_read_error(expr, "expected (call <name> [<deref>] (<param> ...))");
      return NULL;
   }

   exec_list parameters;

   foreach_in_list(s_expression, e, &params->subexpressions) {
      ir_rvalue *param = read_rvalue(e);
      if (param == NULL) {
	 ir_read_error(e, "when reading parameter to function call");
	 return NULL;
      }
      parameters.push_tail(param);
   }

   ir_function *f = state->symbols->get_function(name->value());
   if (f == NULL) {
      ir_read_error(expr, "found call to undefined function %s",
		    name->value());
      return NULL;
   }

   ir_function_signature *callee =
      f->matching_signature(state, &parameters, true);
   if (callee == NULL) {
      ir_read_error(expr, "couldn't find matching signature for function "
                    "%s", name->value());
      return NULL;
   }

   if (callee->return_type == glsl_type::void_type && return_deref) {
      ir_read_error(expr, "call has return value storage but void type");
      return NULL;
   } else if (callee->return_type != glsl_type::void_type && !return_deref) {
      ir_read_error(expr, "call has non-void type but no return value storage");
      return NULL;
   }

   return new(mem_ctx) ir_call(callee, return_deref, &parameters);
}

ir_expression *
ir_reader::read_expression(s_expression *expr)
{
   s_expression *s_type;
   s_symbol *s_op;
   s_expression *s_arg[4] = {NULL};

   s_pattern pat[] = { "expression", s_type, s_op, s_arg[0] };
   if (!PARTIAL_MATCH(expr, pat)) {
      ir_read_error(expr, "expected (expression <type> <operator> "
			  "<operand> [<operand>] [<operand>] [<operand>])");
      return NULL;
   }
   s_arg[1] = (s_expression *) s_arg[0]->next; // may be tail sentinel
   s_arg[2] = (s_expression *) s_arg[1]->next; // may be tail sentinel or NULL
   if (s_arg[2])
      s_arg[3] = (s_expression *) s_arg[2]->next; // may be tail sentinel or NULL

   const glsl_type *type = read_type(s_type);
   if (type == NULL)
      return NULL;

   /* Read the operator */
   ir_expression_operation op = ir_expression::get_operator(s_op->value());
   if (op == (ir_expression_operation) -1) {
      ir_read_error(expr, "invalid operator: %s", s_op->value());
      return NULL;
   }

   /* Skip "expression" <type> <operation> by subtracting 3. */
   int num_operands = (int) ((s_list *) expr)->subexpressions.length() - 3;

   int expected_operands = ir_expression::get_num_operands(op);
   if (num_operands != expected_operands) {
      ir_read_error(expr, "found %d expression operands, expected %d",
                    num_operands, expected_operands);
      return NULL;
   }

   ir_rvalue *arg[4] = {NULL};
   for (int i = 0; i < num_operands; i++) {
      arg[i] = read_rvalue(s_arg[i]);
      if (arg[i] == NULL) {
         ir_read_error(NULL, "when reading operand #%d of %s", i, s_op->value());
         return NULL;
      }
   }

   return new(mem_ctx) ir_expression(op, type, arg[0], arg[1], arg[2], arg[3]);
}

ir_swizzle *
ir_reader::read_swizzle(s_expression *expr)
{
   s_symbol *swiz;
   s_expression *sub;

   s_pattern pat[] = { "swiz", swiz, sub };
   if (!MATCH(expr, pat)) {
      ir_read_error(expr, "expected (swiz <swizzle> <rvalue>)");
      return NULL;
   }

   if (strlen(swiz->value()) > 4) {
      ir_read_error(expr, "expected a valid swizzle; found %s", swiz->value());
      return NULL;
   }

   ir_rvalue *rvalue = read_rvalue(sub);
   if (rvalue == NULL)
      return NULL;

   ir_swizzle *ir = ir_swizzle::create(rvalue, swiz->value(),
				       rvalue->type->vector_elements);
   if (ir == NULL)
      ir_read_error(expr, "invalid swizzle");

   return ir;
}

ir_constant *
ir_reader::read_constant(s_expression *expr)
{
   s_expression *type_expr;
   s_list *values;

   s_pattern pat[] = { "constant", type_expr, values };
   if (!MATCH(expr, pat)) {
      ir_read_error(expr, "expected (constant <type> (...))");
      return NULL;
   }

   const glsl_type *type = read_type(type_expr);
   if (type == NULL)
      return NULL;

   if (values == NULL) {
      ir_read_error(expr, "expected (constant <type> (...))");
      return NULL;
   }

   if (type->is_array()) {
      unsigned elements_supplied = 0;
      exec_list elements;
      foreach_in_list(s_expression, elt, &values->subexpressions) {
	 ir_constant *ir_elt = read_constant(elt);
	 if (ir_elt == NULL)
	    return NULL;
	 elements.push_tail(ir_elt);
	 elements_supplied++;
      }

      if (elements_supplied != type->length) {
	 ir_read_error(values, "expected exactly %u array elements, "
		       "given %u", type->length, elements_supplied);
	 return NULL;
      }
      return new(mem_ctx) ir_constant(type, &elements);
   }

   ir_constant_data data = { { 0 } };

   // Read in list of values (at most 16).
   unsigned k = 0;
   foreach_in_list(s_expression, expr, &values->subexpressions) {
      if (k >= 16) {
	 ir_read_error(values, "expected at most 16 numbers");
	 return NULL;
      }

      if (type->is_float()) {
	 s_number *value = SX_AS_NUMBER(expr);
	 if (value == NULL) {
	    ir_read_error(values, "expected numbers");
	    return NULL;
	 }
	 data.f[k] = value->fvalue();
      } else {
	 s_int *value = SX_AS_INT(expr);
	 if (value == NULL) {
	    ir_read_error(values, "expected integers");
	    return NULL;
	 }

	 switch (type->base_type) {
	 case GLSL_TYPE_UINT: {
	    data.u[k] = value->value();
	    break;
	 }
	 case GLSL_TYPE_INT: {
	    data.i[k] = value->value();
	    break;
	 }
	 case GLSL_TYPE_BOOL: {
	    data.b[k] = value->value();
	    break;
	 }
	 default:
	    ir_read_error(values, "unsupported constant type");
	    return NULL;
	 }
      }
      ++k;
   }
   if (k != type->components()) {
      ir_read_error(values, "expected %u constant values, found %u",
		    type->components(), k);
      return NULL;
   }

   return new(mem_ctx) ir_constant(type, &data);
}

ir_dereference_variable *
ir_reader::read_var_ref(s_expression *expr)
{
   s_symbol *s_var;
   s_pattern var_pat[] = { "var_ref", s_var };

   if (MATCH(expr, var_pat)) {
      ir_variable *var = state->symbols->get_variable(s_var->value());
      if (var == NULL) {
	 ir_read_error(expr, "undeclared variable: %s", s_var->value());
	 return NULL;
      }
      return new(mem_ctx) ir_dereference_variable(var);
   }
   return NULL;
}

ir_dereference *
ir_reader::read_dereference(s_expression *expr)
{
   s_expression *s_subject;
   s_expression *s_index;
   s_symbol *s_field;

   s_pattern array_pat[] = { "array_ref", s_subject, s_index };
   s_pattern record_pat[] = { "record_ref", s_subject, s_field };

   ir_dereference_variable *var_ref = read_var_ref(expr);
   if (var_ref != NULL) {
      return var_ref;
   } else if (MATCH(expr, array_pat)) {
      ir_rvalue *subject = read_rvalue(s_subject);
      if (subject == NULL) {
	 ir_read_error(NULL, "when reading the subject of an array_ref");
	 return NULL;
      }

      ir_rvalue *idx = read_rvalue(s_index);
      if (idx == NULL) {
	 ir_read_error(NULL, "when reading the index of an array_ref");
	 return NULL;
      }
      return new(mem_ctx) ir_dereference_array(subject, idx);
   } else if (MATCH(expr, record_pat)) {
      ir_rvalue *subject = read_rvalue(s_subject);
      if (subject == NULL) {
	 ir_read_error(NULL, "when reading the subject of a record_ref");
	 return NULL;
      }
      return new(mem_ctx) ir_dereference_record(subject, s_field->value());
   }
   return NULL;
}

ir_texture *
ir_reader::read_texture(s_expression *expr)
{
   s_symbol *tag = NULL;
   s_expression *s_type = NULL;
   s_expression *s_sampler = NULL;
   s_expression *s_coord = NULL;
   s_expression *s_offset = NULL;
   s_expression *s_proj = NULL;
   s_list *s_shadow = NULL;
   s_expression *s_lod = NULL;
   s_expression *s_sample_index = NULL;
   s_expression *s_component = NULL;

   ir_texture_opcode op = ir_tex; /* silence warning */

   s_pattern tex_pattern[] =
      { "tex", s_type, s_sampler, s_coord, s_offset, s_proj, s_shadow };
   s_pattern lod_pattern[] =
      { "lod", s_type, s_sampler, s_coord };
   s_pattern txf_pattern[] =
      { "txf", s_type, s_sampler, s_coord, s_offset, s_lod };
   s_pattern txf_ms_pattern[] =
      { "txf_ms", s_type, s_sampler, s_coord, s_sample_index };
   s_pattern txs_pattern[] =
      { "txs", s_type, s_sampler, s_lod };
   s_pattern tg4_pattern[] =
      { "tg4", s_type, s_sampler, s_coord, s_offset, s_component };
   s_pattern query_levels_pattern[] =
      { "query_levels", s_type, s_sampler };
   s_pattern texture_samples_pattern[] =
      { "samples", s_type, s_sampler };
   s_pattern other_pattern[] =
      { tag, s_type, s_sampler, s_coord, s_offset, s_proj, s_shadow, s_lod };

   if (MATCH(expr, lod_pattern)) {
      op = ir_lod;
   } else if (MATCH(expr, tex_pattern)) {
      op = ir_tex;
   } else if (MATCH(expr, txf_pattern)) {
      op = ir_txf;
   } else if (MATCH(expr, txf_ms_pattern)) {
      op = ir_txf_ms;
   } else if (MATCH(expr, txs_pattern)) {
      op = ir_txs;
   } else if (MATCH(expr, tg4_pattern)) {
      op = ir_tg4;
   } else if (MATCH(expr, query_levels_pattern)) {
      op = ir_query_levels;
   } else if (MATCH(expr, texture_samples_pattern)) {
      op = ir_texture_samples;
   } else if (MATCH(expr, other_pattern)) {
      op = ir_texture::get_opcode(tag->value());
      if (op == (ir_texture_opcode) -1)
	 return NULL;
   } else {
      ir_read_error(NULL, "unexpected texture pattern %s", tag->value());
      return NULL;
   }

   ir_texture *tex = new(mem_ctx) ir_texture(op);

   // Read return type
   const glsl_type *type = read_type(s_type);
   if (type == NULL) {
      ir_read_error(NULL, "when reading type in (%s ...)",
		    tex->opcode_string());
      return NULL;
   }

   // Read sampler (must be a deref)
   ir_dereference *sampler = read_dereference(s_sampler);
   if (sampler == NULL) {
      ir_read_error(NULL, "when reading sampler in (%s ...)",
		    tex->opcode_string());
      return NULL;
   }
   tex->set_sampler(sampler, type);

   if (op != ir_txs) {
      // Read coordinate (any rvalue)
      tex->coordinate = read_rvalue(s_coord);
      if (tex->coordinate == NULL) {
	 ir_read_error(NULL, "when reading coordinate in (%s ...)",
		       tex->opcode_string());
	 return NULL;
      }

      if (op != ir_txf_ms && op != ir_lod) {
         // Read texel offset - either 0 or an rvalue.
         s_int *si_offset = SX_AS_INT(s_offset);
         if (si_offset == NULL || si_offset->value() != 0) {
            tex->offset = read_rvalue(s_offset);
            if (tex->offset == NULL) {
               ir_read_error(s_offset, "expected 0 or an expression");
               return NULL;
            }
         }
      }
   }

   if (op != ir_txf && op != ir_txf_ms &&
       op != ir_txs && op != ir_lod && op != ir_tg4 &&
       op != ir_query_levels && op != ir_texture_samples) {
      s_int *proj_as_int = SX_AS_INT(s_proj);
      if (proj_as_int && proj_as_int->value() == 1) {
	 tex->projector = NULL;
      } else {
	 tex->projector = read_rvalue(s_proj);
	 if (tex->projector == NULL) {
	    ir_read_error(NULL, "when reading projective divide in (%s ..)",
	                  tex->opcode_string());
	    return NULL;
	 }
      }

      if (s_shadow->subexpressions.is_empty()) {
	 tex->shadow_comparator = NULL;
      } else {
	 tex->shadow_comparator = read_rvalue(s_shadow);
	 if (tex->shadow_comparator == NULL) {
	    ir_read_error(NULL, "when reading shadow comparator in (%s ..)",
			  tex->opcode_string());
	    return NULL;
	 }
      }
   }

   switch (op) {
   case ir_txb:
      tex->lod_info.bias = read_rvalue(s_lod);
      if (tex->lod_info.bias == NULL) {
	 ir_read_error(NULL, "when reading LOD bias in (txb ...)");
	 return NULL;
      }
      break;
   case ir_txl:
   case ir_txf:
   case ir_txs:
      tex->lod_info.lod = read_rvalue(s_lod);
      if (tex->lod_info.lod == NULL) {
	 ir_read_error(NULL, "when reading LOD in (%s ...)",
		       tex->opcode_string());
	 return NULL;
      }
      break;
   case ir_txf_ms:
      tex->lod_info.sample_index = read_rvalue(s_sample_index);
      if (tex->lod_info.sample_index == NULL) {
         ir_read_error(NULL, "when reading sample_index in (txf_ms ...)");
         return NULL;
      }
      break;
   case ir_txd: {
      s_expression *s_dx, *s_dy;
      s_pattern dxdy_pat[] = { s_dx, s_dy };
      if (!MATCH(s_lod, dxdy_pat)) {
	 ir_read_error(s_lod, "expected (dPdx dPdy) in (txd ...)");
	 return NULL;
      }
      tex->lod_info.grad.dPdx = read_rvalue(s_dx);
      if (tex->lod_info.grad.dPdx == NULL) {
	 ir_read_error(NULL, "when reading dPdx in (txd ...)");
	 return NULL;
      }
      tex->lod_info.grad.dPdy = read_rvalue(s_dy);
      if (tex->lod_info.grad.dPdy == NULL) {
	 ir_read_error(NULL, "when reading dPdy in (txd ...)");
	 return NULL;
      }
      break;
   }
   case ir_tg4:
      tex->lod_info.component = read_rvalue(s_component);
      if (tex->lod_info.component == NULL) {
         ir_read_error(NULL, "when reading component in (tg4 ...)");
         return NULL;
      }
      break;
   default:
      // tex and lod don't have any extra parameters.
      break;
   };
   return tex;
}

ir_emit_vertex *
ir_reader::read_emit_vertex(s_expression *expr)
{
   s_expression *s_stream = NULL;

   s_pattern pat[] = { "emit-vertex", s_stream };

   if (MATCH(expr, pat)) {
      ir_rvalue *stream = read_dereference(s_stream);
      if (stream == NULL) {
         ir_read_error(NULL, "when reading stream info in emit-vertex");
         return NULL;
      }
      return new(mem_ctx) ir_emit_vertex(stream);
   }
   ir_read_error(NULL, "when reading emit-vertex");
   return NULL;
}

ir_end_primitive *
ir_reader::read_end_primitive(s_expression *expr)
{
   s_expression *s_stream = NULL;

   s_pattern pat[] = { "end-primitive", s_stream };

   if (MATCH(expr, pat)) {
      ir_rvalue *stream = read_dereference(s_stream);
      if (stream == NULL) {
         ir_read_error(NULL, "when reading stream info in end-primitive");
         return NULL;
      }
      return new(mem_ctx) ir_end_primitive(stream);
   }
   ir_read_error(NULL, "when reading end-primitive");
   return NULL;
}

ir_barrier *
ir_reader::read_barrier(s_expression *expr)
{
   s_pattern pat[] = { "barrier" };

   if (MATCH(expr, pat)) {
      return new(mem_ctx) ir_barrier();
   }
   ir_read_error(NULL, "when reading barrier");
   return NULL;
}
