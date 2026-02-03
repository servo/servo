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

#include <inttypes.h> /* for PRIx64 macro */
#include "ir_print_visitor.h"
#include "compiler/glsl_types.h"
#include "glsl_parser_extras.h"
#include "main/macros.h"
#include "util/hash_table.h"
#include "util/u_string.h"
#include "util/half_float.h"

static void print_type(FILE *f, const glsl_type *t);

void
ir_instruction::print(void) const
{
   this->fprint(stdout);
}

void
ir_instruction::fprint(FILE *f) const
{
   ir_instruction *deconsted = const_cast<ir_instruction *>(this);

   ir_print_visitor v(f);
   deconsted->accept(&v);
}

extern "C" {
void
_mesa_print_ir(FILE *f, exec_list *instructions,
	       struct _mesa_glsl_parse_state *state)
{
   if (state) {
      for (unsigned i = 0; i < state->num_user_structures; i++) {
	 const glsl_type *const s = state->user_structures[i];

	 fprintf(f, "(structure (%s) (%s@%p) (%u) (\n",
                 s->name, s->name, (void *) s, s->length);

	 for (unsigned j = 0; j < s->length; j++) {
	    fprintf(f, "\t((");
	    print_type(f, s->fields.structure[j].type);
	    fprintf(f, ")(%s))\n", s->fields.structure[j].name);
	 }

	 fprintf(f, ")\n");
      }
   }

   fprintf(f, "(\n");
   foreach_in_list(ir_instruction, ir, instructions) {
      ir->fprint(f);
      if (ir->ir_type != ir_type_function)
	 fprintf(f, "\n");
   }
   fprintf(f, ")\n");
}

void
fprint_ir(FILE *f, const void *instruction)
{
   const ir_instruction *ir = (const ir_instruction *)instruction;
   ir->fprint(f);
}

} /* extern "C" */

ir_print_visitor::ir_print_visitor(FILE *f)
   : f(f)
{
   indentation = 0;
   printable_names = _mesa_pointer_hash_table_create(NULL);
   symbols = _mesa_symbol_table_ctor();
   mem_ctx = ralloc_context(NULL);
}

ir_print_visitor::~ir_print_visitor()
{
   _mesa_hash_table_destroy(printable_names, NULL);
   _mesa_symbol_table_dtor(symbols);
   ralloc_free(mem_ctx);
}

void ir_print_visitor::indent(void)
{
   for (int i = 0; i < indentation; i++)
      fprintf(f, "  ");
}

const char *
ir_print_visitor::unique_name(ir_variable *var)
{
   /* var->name can be NULL in function prototypes when a type is given for a
    * parameter but no name is given.  In that case, just return an empty
    * string.  Don't worry about tracking the generated name in the printable
    * names hash because this is the only scope where it can ever appear.
    */
   if (var->name == NULL) {
      static unsigned arg = 1;
      return ralloc_asprintf(this->mem_ctx, "parameter@%u", arg++);
   }

   /* Do we already have a name for this variable? */
   struct hash_entry * entry =
      _mesa_hash_table_search(this->printable_names, var);

   if (entry != NULL) {
      return (const char *) entry->data;
   }

   /* If there's no conflict, just use the original name */
   const char* name = NULL;
   if (_mesa_symbol_table_find_symbol(this->symbols, var->name) == NULL) {
      name = var->name;
   } else {
      static unsigned i = 1;
      name = ralloc_asprintf(this->mem_ctx, "%s@%u", var->name, ++i);
   }
   _mesa_hash_table_insert(this->printable_names, var, (void *) name);
   _mesa_symbol_table_add_symbol(this->symbols, name, var);
   return name;
}

static void
print_type(FILE *f, const glsl_type *t)
{
   if (t->is_array()) {
      fprintf(f, "(array ");
      print_type(f, t->fields.array);
      fprintf(f, " %u)", t->length);
   } else if (t->is_struct() && !is_gl_identifier(t->name)) {
      fprintf(f, "%s@%p", t->name, (void *) t);
   } else {
      fprintf(f, "%s", t->name);
   }
}

void ir_print_visitor::visit(ir_rvalue *)
{
   fprintf(f, "error");
}

void ir_print_visitor::visit(ir_variable *ir)
{
   fprintf(f, "(declare ");

   char binding[32] = {0};
   if (ir->data.binding)
      snprintf(binding, sizeof(binding), "binding=%i ", ir->data.binding);

   char loc[32] = {0};
   if (ir->data.location != -1)
      snprintf(loc, sizeof(loc), "location=%i ", ir->data.location);

   char component[32] = {0};
   if (ir->data.explicit_component || ir->data.location_frac != 0)
      snprintf(component, sizeof(component), "component=%i ",
                    ir->data.location_frac);

   char stream[32] = {0};
   if (ir->data.stream & (1u << 31)) {
      if (ir->data.stream & ~(1u << 31)) {
         snprintf(stream, sizeof(stream), "stream(%u,%u,%u,%u) ",
                  ir->data.stream & 3, (ir->data.stream >> 2) & 3,
                  (ir->data.stream >> 4) & 3, (ir->data.stream >> 6) & 3);
      }
   } else if (ir->data.stream) {
      snprintf(stream, sizeof(stream), "stream%u ", ir->data.stream);
   }

   char image_format[32] = {0};
   if (ir->data.image_format) {
      snprintf(image_format, sizeof(image_format), "format=%x ",
                    ir->data.image_format);
   }

   const char *const cent = (ir->data.centroid) ? "centroid " : "";
   const char *const samp = (ir->data.sample) ? "sample " : "";
   const char *const patc = (ir->data.patch) ? "patch " : "";
   const char *const inv = (ir->data.invariant) ? "invariant " : "";
   const char *const explicit_inv = (ir->data.explicit_invariant) ? "explicit_invariant " : "";
   const char *const prec = (ir->data.precise) ? "precise " : "";
   const char *const bindless = (ir->data.bindless) ? "bindless " : "";
   const char *const bound = (ir->data.bound) ? "bound " : "";
   const char *const memory_read_only = (ir->data.memory_read_only) ? "readonly " : "";
   const char *const memory_write_only = (ir->data.memory_write_only) ? "writeonly " : "";
   const char *const memory_coherent = (ir->data.memory_coherent) ? "coherent " : "";
   const char *const memory_volatile = (ir->data.memory_volatile) ? "volatile " : "";
   const char *const memory_restrict = (ir->data.memory_restrict) ? "restrict " : "";
   const char *const mode[] = { "", "uniform ", "shader_storage ",
                                "shader_shared ", "shader_in ", "shader_out ",
                                "in ", "out ", "inout ",
			        "const_in ", "sys ", "temporary " };
   STATIC_ASSERT(ARRAY_SIZE(mode) == ir_var_mode_count);
   const char *const interp[] = { "", "smooth", "flat", "noperspective", "explicit" };
   STATIC_ASSERT(ARRAY_SIZE(interp) == INTERP_MODE_COUNT);

   fprintf(f, "(%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s) ",
           binding, loc, component, cent, bindless, bound,
           image_format, memory_read_only, memory_write_only,
           memory_coherent, memory_volatile, memory_restrict,
           samp, patc, inv, explicit_inv, prec, mode[ir->data.mode],
           stream,
           interp[ir->data.interpolation]);

   print_type(f, ir->type);
   fprintf(f, " %s)", unique_name(ir));
}


void ir_print_visitor::visit(ir_function_signature *ir)
{
   _mesa_symbol_table_push_scope(symbols);
   fprintf(f, "(signature ");
   indentation++;

   print_type(f, ir->return_type);
   fprintf(f, "\n");
   indent();

   fprintf(f, "(parameters\n");
   indentation++;

   foreach_in_list(ir_variable, inst, &ir->parameters) {
      indent();
      inst->accept(this);
      fprintf(f, "\n");
   }
   indentation--;

   indent();
   fprintf(f, ")\n");

   indent();

   fprintf(f, "(\n");
   indentation++;

   foreach_in_list(ir_instruction, inst, &ir->body) {
      indent();
      inst->accept(this);
      fprintf(f, "\n");
   }
   indentation--;
   indent();
   fprintf(f, "))\n");
   indentation--;
   _mesa_symbol_table_pop_scope(symbols);
}


void ir_print_visitor::visit(ir_function *ir)
{
   fprintf(f, "(%s function %s\n", ir->is_subroutine ? "subroutine" : "", ir->name);
   indentation++;
   foreach_in_list(ir_function_signature, sig, &ir->signatures) {
      indent();
      sig->accept(this);
      fprintf(f, "\n");
   }
   indentation--;
   indent();
   fprintf(f, ")\n\n");
}


void ir_print_visitor::visit(ir_expression *ir)
{
   fprintf(f, "(expression ");

   print_type(f, ir->type);

   fprintf(f, " %s ", ir_expression_operation_strings[ir->operation]);

   for (unsigned i = 0; i < ir->num_operands; i++) {
      ir->operands[i]->accept(this);
   }

   fprintf(f, ") ");
}


void ir_print_visitor::visit(ir_texture *ir)
{
   fprintf(f, "(%s ", ir->opcode_string());

   if (ir->op == ir_samples_identical) {
      ir->sampler->accept(this);
      fprintf(f, " ");
      ir->coordinate->accept(this);
      fprintf(f, ")");
      return;
   }

   print_type(f, ir->type);
   fprintf(f, " ");

   ir->sampler->accept(this);
   fprintf(f, " ");

   if (ir->op != ir_txs && ir->op != ir_query_levels &&
       ir->op != ir_texture_samples) {
      ir->coordinate->accept(this);

      fprintf(f, " ");

      if (ir->offset != NULL) {
	 ir->offset->accept(this);
      } else {
	 fprintf(f, "0");
      }

      fprintf(f, " ");
   }

   if (ir->op != ir_txf && ir->op != ir_txf_ms &&
       ir->op != ir_txs && ir->op != ir_tg4 &&
       ir->op != ir_query_levels && ir->op != ir_texture_samples) {
      if (ir->projector)
	 ir->projector->accept(this);
      else
	 fprintf(f, "1");

      if (ir->shadow_comparator) {
	 fprintf(f, " ");
	 ir->shadow_comparator->accept(this);
      } else {
	 fprintf(f, " ()");
      }
   }

   fprintf(f, " ");
   switch (ir->op)
   {
   case ir_tex:
   case ir_lod:
   case ir_query_levels:
   case ir_texture_samples:
      break;
   case ir_txb:
      ir->lod_info.bias->accept(this);
      break;
   case ir_txl:
   case ir_txf:
   case ir_txs:
      ir->lod_info.lod->accept(this);
      break;
   case ir_txf_ms:
      ir->lod_info.sample_index->accept(this);
      break;
   case ir_txd:
      fprintf(f, "(");
      ir->lod_info.grad.dPdx->accept(this);
      fprintf(f, " ");
      ir->lod_info.grad.dPdy->accept(this);
      fprintf(f, ")");
      break;
   case ir_tg4:
      ir->lod_info.component->accept(this);
      break;
   case ir_samples_identical:
      unreachable("ir_samples_identical was already handled");
   };
   fprintf(f, ")");
}


void ir_print_visitor::visit(ir_swizzle *ir)
{
   const unsigned swiz[4] = {
      ir->mask.x,
      ir->mask.y,
      ir->mask.z,
      ir->mask.w,
   };

   fprintf(f, "(swiz ");
   for (unsigned i = 0; i < ir->mask.num_components; i++) {
      fprintf(f, "%c", "xyzw"[swiz[i]]);
   }
   fprintf(f, " ");
   ir->val->accept(this);
   fprintf(f, ")");
}


void ir_print_visitor::visit(ir_dereference_variable *ir)
{
   ir_variable *var = ir->variable_referenced();
   fprintf(f, "(var_ref %s) ", unique_name(var));
}


void ir_print_visitor::visit(ir_dereference_array *ir)
{
   fprintf(f, "(array_ref ");
   ir->array->accept(this);
   ir->array_index->accept(this);
   fprintf(f, ") ");
}


void ir_print_visitor::visit(ir_dereference_record *ir)
{
   fprintf(f, "(record_ref ");
   ir->record->accept(this);

   const char *field_name =
      ir->record->type->fields.structure[ir->field_idx].name;
   fprintf(f, " %s) ", field_name);
}


void ir_print_visitor::visit(ir_assignment *ir)
{
   fprintf(f, "(assign ");

   if (ir->condition)
      ir->condition->accept(this);

   char mask[5];
   unsigned j = 0;

   for (unsigned i = 0; i < 4; i++) {
      if ((ir->write_mask & (1 << i)) != 0) {
	 mask[j] = "xyzw"[i];
	 j++;
      }
   }
   mask[j] = '\0';

   fprintf(f, " (%s) ", mask);

   ir->lhs->accept(this);

   fprintf(f, " ");

   ir->rhs->accept(this);
   fprintf(f, ") ");
}

static void
print_float_constant(FILE *f, float val)
{
   if (val == 0.0f)
      /* 0.0 == -0.0, so print with %f to get the proper sign. */
      fprintf(f, "%f", val);
   else if (fabs(val) < 0.000001f)
      fprintf(f, "%a", val);
   else if (fabs(val) > 1000000.0f)
      fprintf(f, "%e", val);
   else
      fprintf(f, "%f", val);
}

void ir_print_visitor::visit(ir_constant *ir)
{
   fprintf(f, "(constant ");
   print_type(f, ir->type);
   fprintf(f, " (");

   if (ir->type->is_array()) {
      for (unsigned i = 0; i < ir->type->length; i++)
	 ir->get_array_element(i)->accept(this);
   } else if (ir->type->is_struct()) {
      for (unsigned i = 0; i < ir->type->length; i++) {
	 fprintf(f, "(%s ", ir->type->fields.structure[i].name);
         ir->get_record_field(i)->accept(this);
	 fprintf(f, ")");
      }
   } else {
      for (unsigned i = 0; i < ir->type->components(); i++) {
	 if (i != 0)
	    fprintf(f, " ");
	 switch (ir->type->base_type) {
	 case GLSL_TYPE_UINT:  fprintf(f, "%u", ir->value.u[i]); break;
	 case GLSL_TYPE_INT:   fprintf(f, "%d", ir->value.i[i]); break;
	 case GLSL_TYPE_FLOAT:
            print_float_constant(f, ir->value.f[i]);
            break;
	 case GLSL_TYPE_FLOAT16:
            print_float_constant(f, _mesa_half_to_float(ir->value.f16[i]));
            break;
	 case GLSL_TYPE_SAMPLER:
	 case GLSL_TYPE_IMAGE:
	 case GLSL_TYPE_UINT64:
            fprintf(f, "%" PRIu64, ir->value.u64[i]);
            break;
	 case GLSL_TYPE_INT64: fprintf(f, "%" PRIi64, ir->value.i64[i]); break;
	 case GLSL_TYPE_BOOL:  fprintf(f, "%d", ir->value.b[i]); break;
	 case GLSL_TYPE_DOUBLE:
            if (ir->value.d[i] == 0.0)
               /* 0.0 == -0.0, so print with %f to get the proper sign. */
               fprintf(f, "%.1f", ir->value.d[i]);
            else if (fabs(ir->value.d[i]) < 0.000001)
               fprintf(f, "%a", ir->value.d[i]);
            else if (fabs(ir->value.d[i]) > 1000000.0)
               fprintf(f, "%e", ir->value.d[i]);
            else
               fprintf(f, "%f", ir->value.d[i]);
            break;
	 default:
            unreachable("Invalid constant type");
	 }
      }
   }
   fprintf(f, ")) ");
}


void
ir_print_visitor::visit(ir_call *ir)
{
   fprintf(f, "(call %s ", ir->callee_name());
   if (ir->return_deref)
      ir->return_deref->accept(this);
   fprintf(f, " (");
   foreach_in_list(ir_rvalue, param, &ir->actual_parameters) {
      param->accept(this);
   }
   fprintf(f, "))\n");
}


void
ir_print_visitor::visit(ir_return *ir)
{
   fprintf(f, "(return");

   ir_rvalue *const value = ir->get_value();
   if (value) {
      fprintf(f, " ");
      value->accept(this);
   }

   fprintf(f, ")");
}


void
ir_print_visitor::visit(ir_discard *ir)
{
   fprintf(f, "(discard ");

   if (ir->condition != NULL) {
      fprintf(f, " ");
      ir->condition->accept(this);
   }

   fprintf(f, ")");
}


void
ir_print_visitor::visit(ir_demote *ir)
{
   fprintf(f, "(demote)");
}


void
ir_print_visitor::visit(ir_if *ir)
{
   fprintf(f, "(if ");
   ir->condition->accept(this);

   fprintf(f, "(\n");
   indentation++;

   foreach_in_list(ir_instruction, inst, &ir->then_instructions) {
      indent();
      inst->accept(this);
      fprintf(f, "\n");
   }

   indentation--;
   indent();
   fprintf(f, ")\n");

   indent();
   if (!ir->else_instructions.is_empty()) {
      fprintf(f, "(\n");
      indentation++;

      foreach_in_list(ir_instruction, inst, &ir->else_instructions) {
	 indent();
	 inst->accept(this);
	 fprintf(f, "\n");
      }
      indentation--;
      indent();
      fprintf(f, "))\n");
   } else {
      fprintf(f, "())\n");
   }
}


void
ir_print_visitor::visit(ir_loop *ir)
{
   fprintf(f, "(loop (\n");
   indentation++;

   foreach_in_list(ir_instruction, inst, &ir->body_instructions) {
      indent();
      inst->accept(this);
      fprintf(f, "\n");
   }
   indentation--;
   indent();
   fprintf(f, "))\n");
}


void
ir_print_visitor::visit(ir_loop_jump *ir)
{
   fprintf(f, "%s", ir->is_break() ? "break" : "continue");
}

void
ir_print_visitor::visit(ir_precision_statement *ir)
{
	//printf("%s", ir->precision_statement);
}

void
ir_print_visitor::visit(ir_typedecl_statement *)
{
}

void
ir_print_visitor::visit(ir_emit_vertex *ir)
{
   fprintf(f, "(emit-vertex ");
   ir->stream->accept(this);
   fprintf(f, ")\n");
}

void
ir_print_visitor::visit(ir_end_primitive *ir)
{
   fprintf(f, "(end-primitive ");
   ir->stream->accept(this);
   fprintf(f, ")\n");
}

void
ir_print_visitor::visit(ir_barrier *)
{
   fprintf(f, "(barrier)\n");
}
