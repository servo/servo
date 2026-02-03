/*
 * Copyright © 2010 Luca Barbieri
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
 * \file lower_variable_index_to_cond_assign.cpp
 *
 * Turns non-constant indexing into array types to a series of
 * conditional moves of each element into a temporary.
 *
 * Pre-DX10 GPUs often don't have a native way to do this operation,
 * and this works around that.
 *
 * The lowering process proceeds as follows.  Each non-constant index
 * found in an r-value is converted to a canonical form \c array[i].  Each
 * element of the array is conditionally assigned to a temporary by comparing
 * \c i to a constant index.  This is done by cloning the canonical form and
 * replacing all occurances of \c i with a constant.  Each remaining occurance
 * of the canonical form in the IR is replaced with a dereference of the
 * temporary variable.
 *
 * L-values with non-constant indices are handled similarly.  In this case,
 * the RHS of the assignment is assigned to a temporary.  The non-constant
 * index is replace with the canonical form (just like for r-values).  The
 * temporary is conditionally assigned to each element of the canonical form
 * by comparing \c i with each index.  The same clone-and-replace scheme is
 * used.
 */

#include "ir.h"
#include "ir_rvalue_visitor.h"
#include "ir_optimization.h"
#include "compiler/glsl_types.h"
#include "main/macros.h"
#include "program/prog_instruction.h" /* For SWIZZLE_XXXX */
#include "ir_builder.h"

using namespace ir_builder;

/**
 * Generate a comparison value for a block of indices
 *
 * Lowering passes for non-constant indexing of arrays, matrices, or vectors
 * can use this to generate blocks of index comparison values.
 *
 * \param instructions  List where new instructions will be appended
 * \param index         \c ir_variable containing the desired index
 * \param base          Base value for this block of comparisons
 * \param components    Number of unique index values to compare.  This must
 *                      be on the range [1, 4].
 * \param mem_ctx       ralloc memory context to be used for all allocations.
 *
 * \returns
 * An \c ir_variable containing the per-component comparison results.  This
 * must be dereferenced per use.
 */
ir_variable *
compare_index_block(ir_factory &body, ir_variable *index,
                    unsigned base, unsigned components)
{
   assert(index->type->is_scalar());
   assert(index->type->base_type == GLSL_TYPE_INT ||
          index->type->base_type == GLSL_TYPE_UINT);
   assert(components >= 1 && components <= 4);

   ir_rvalue *const broadcast_index = components > 1
      ? swizzle(index, SWIZZLE_XXXX, components)
      : operand(index).val;

   /* Compare the desired index value with the next block of four indices.
    */
   ir_constant_data test_indices_data;
   memset(&test_indices_data, 0, sizeof(test_indices_data));
   test_indices_data.i[0] = base;
   test_indices_data.i[1] = base + 1;
   test_indices_data.i[2] = base + 2;
   test_indices_data.i[3] = base + 3;

   ir_constant *const test_indices =
      new(body.mem_ctx) ir_constant(broadcast_index->type, &test_indices_data);

   ir_rvalue *const condition_val = equal(broadcast_index, test_indices);

   ir_variable *const condition = body.make_temp(condition_val->type,
                                                 "dereference_condition");

   body.emit(assign(condition, condition_val));

   return condition;
}

static inline bool
is_array_or_matrix(const ir_rvalue *ir)
{
   return (ir->type->is_array() || ir->type->is_matrix());
}

namespace {
/**
 * Replace a dereference of a variable with a specified r-value
 *
 * Each time a dereference of the specified value is replaced, the r-value
 * tree is cloned.
 */
class deref_replacer : public ir_rvalue_visitor {
public:
   deref_replacer(const ir_variable *variable_to_replace, ir_rvalue *value)
      : variable_to_replace(variable_to_replace), value(value),
        progress(false)
   {
      assert(this->variable_to_replace != NULL);
      assert(this->value != NULL);
   }

   virtual void handle_rvalue(ir_rvalue **rvalue)
   {
      ir_dereference_variable *const dv = (*rvalue)->as_dereference_variable();

      if (dv != NULL && dv->var == this->variable_to_replace) {
         this->progress = true;
         *rvalue = this->value->clone(ralloc_parent(*rvalue), NULL);
      }
   }

   const ir_variable *variable_to_replace;
   ir_rvalue *value;
   bool progress;
};

/**
 * Find a variable index dereference of an array in an rvalue tree
 */
class find_variable_index : public ir_hierarchical_visitor {
public:
   find_variable_index()
      : deref(NULL)
   {
      /* empty */
   }

   virtual ir_visitor_status visit_enter(ir_dereference_array *ir)
   {
      if (is_array_or_matrix(ir->array) &&
          ir->array_index->as_constant() == NULL) {
         this->deref = ir;
         return visit_stop;
      }

      return visit_continue;
   }

   /**
    * First array dereference found in the tree that has a non-constant index.
    */
   ir_dereference_array *deref;
};

struct assignment_generator
{
   ir_instruction* base_ir;
   ir_dereference *rvalue;
   ir_variable *old_index;
   bool is_write;
   unsigned int write_mask;
   ir_variable* var;

   assignment_generator()
      : base_ir(NULL),
        rvalue(NULL),
        old_index(NULL),
        is_write(false),
        write_mask(0),
        var(NULL)
   {
   }

   void generate(unsigned i, ir_rvalue* condition, ir_factory &body) const
   {
      /* Clone the old r-value in its entirety.  Then replace any occurances of
       * the old variable index with the new constant index.
       */
      ir_dereference *element = this->rvalue->clone(body.mem_ctx, NULL);
      ir_constant *const index = body.constant(i);
      deref_replacer r(this->old_index, index);
      element->accept(&r);
      assert(r.progress);

      /* Generate a conditional assignment to (or from) the constant indexed
       * array dereference.
       */
      ir_assignment *const assignment = (is_write)
         ? assign(element, this->var, condition, write_mask)
         : assign(this->var, element, condition);

      body.emit(assignment);
   }
};

struct switch_generator
{
   /* make TFunction a template parameter if you need to use other generators */
   typedef assignment_generator TFunction;
   const TFunction& generator;

   ir_variable* index;
   unsigned linear_sequence_max_length;
   unsigned condition_components;

   void *mem_ctx;

   switch_generator(const TFunction& generator, ir_variable *index,
                    unsigned linear_sequence_max_length,
                    unsigned condition_components)
      : generator(generator), index(index),
        linear_sequence_max_length(linear_sequence_max_length),
        condition_components(condition_components)
   {
      this->mem_ctx = ralloc_parent(index);
   }

   void linear_sequence(unsigned begin, unsigned end, ir_factory &body)
   {
      if (begin == end)
         return;

      /* If the array access is a read, read the first element of this subregion
       * unconditionally.  The remaining tests will possibly overwrite this
       * value with one of the other array elements.
       *
       * This optimization cannot be done for writes because it will cause the
       * first element of the subregion to be written possibly *in addition* to
       * one of the other elements.
       */
      unsigned first;
      if (!this->generator.is_write) {
         this->generator.generate(begin, 0, body);
         first = begin + 1;
      } else {
         first = begin;
      }

      for (unsigned i = first; i < end; i += 4) {
         const unsigned comps = MIN2(condition_components, end - i);
         ir_variable *const cond = compare_index_block(body, index, i, comps);

         if (comps == 1) {
            this->generator.generate(i,
                                     operand(cond).val,
                                     body);
         } else {
            for (unsigned j = 0; j < comps; j++) {
               this->generator.generate(i + j,
                                        swizzle(cond, j, 1),
                                        body);
            }
         }
      }
   }

   void bisect(unsigned begin, unsigned end, ir_factory &body)
   {
      unsigned middle = (begin + end) >> 1;

      assert(index->type->is_integer_32());

      ir_constant *const middle_c = (index->type->base_type == GLSL_TYPE_UINT)
         ? new(body.mem_ctx) ir_constant((unsigned)middle)
         : new(body.mem_ctx) ir_constant((int)middle);

      ir_if *if_less = new(body.mem_ctx) ir_if(less(this->index, middle_c));

      ir_factory then_body(&if_less->then_instructions, body.mem_ctx);
      ir_factory else_body(&if_less->else_instructions, body.mem_ctx);
      generate(begin, middle, then_body);
      generate(middle, end, else_body);

      body.emit(if_less);
   }

   void generate(unsigned begin, unsigned end, ir_factory &body)
   {
      unsigned length = end - begin;
      if (length <= this->linear_sequence_max_length)
         return linear_sequence(begin, end, body);
      else
         return bisect(begin, end, body);
   }
};

/**
 * Visitor class for replacing expressions with ir_constant values.
 */

class variable_index_to_cond_assign_visitor : public ir_rvalue_visitor {
public:
   variable_index_to_cond_assign_visitor(gl_shader_stage stage,
                                         bool lower_input,
                                         bool lower_output,
                                         bool lower_temp,
                                         bool lower_uniform)
      : progress(false), stage(stage), lower_inputs(lower_input),
        lower_outputs(lower_output), lower_temps(lower_temp),
        lower_uniforms(lower_uniform)
   {
      /* empty */
   }

   bool progress;

   gl_shader_stage stage;
   bool lower_inputs;
   bool lower_outputs;
   bool lower_temps;
   bool lower_uniforms;

   bool storage_type_needs_lowering(ir_dereference_array *deref) const
   {
      /* If a variable isn't eventually the target of this dereference, then
       * it must be a constant or some sort of anonymous temporary storage.
       *
       * FINISHME: Is this correct?  Most drivers treat arrays of constants as
       * FINISHME: uniforms.  It seems like this should do the same.
       */
      const ir_variable *const var = deref->array->variable_referenced();
      if (var == NULL)
         return this->lower_temps;

      switch (var->data.mode) {
      case ir_var_auto:
      case ir_var_temporary:
         return this->lower_temps;

      case ir_var_uniform:
      case ir_var_shader_storage:
         return this->lower_uniforms;

      case ir_var_shader_shared:
         return false;

      case ir_var_function_in:
      case ir_var_const_in:
         return this->lower_temps;

      case ir_var_system_value:
         /* There are only a few system values that have array types:
          *
          *    gl_TessLevelInner[]
          *    gl_TessLevelOuter[]
          *    gl_SampleMaskIn[]
          *
          * The tessellation factor arrays are lowered to vec4/vec2s
          * by lower_tess_level() before this pass occurs, so we'll
          * never see them here.
          *
          * The only remaining case is gl_SampleMaskIn[], which has
          * a length of ceil(ctx->Const.MaxSamples / 32).  Most hardware
          * supports no more than 32 samples, in which case our lowering
          * produces a single read of gl_SampleMaskIn[0].  Even with 64x
          * MSAA, the array length is only 2, so the lowering is fairly
          * efficient.  Therefore, lower unconditionally.
          */
         return true;

      case ir_var_shader_in:
         /* The input array size is unknown at compiler time for non-patch
          * inputs in TCS and TES. The arrays are sized to
          * the implementation-dependent limit "gl_MaxPatchVertices", but
          * the real size is stored in the "gl_PatchVerticesIn" built-in
          * uniform.
          *
          * The TCS input array size is specified by
          * glPatchParameteri(GL_PATCH_VERTICES).
          *
          * The TES input array size is specified by the "vertices" output
          * layout qualifier in TCS.
          */
         if ((stage == MESA_SHADER_TESS_CTRL ||
              stage == MESA_SHADER_TESS_EVAL) && !var->data.patch)
            return false;
         return this->lower_inputs;

      case ir_var_function_out:
         /* TCS non-patch outputs can only be indexed with "gl_InvocationID".
          * Other expressions are not allowed.
          */
         if (stage == MESA_SHADER_TESS_CTRL && !var->data.patch)
            return false;
         return this->lower_temps;

      case ir_var_shader_out:
         return this->lower_outputs;

      case ir_var_function_inout:
         return this->lower_temps;
      }

      assert(!"Should not get here.");
      return false;
   }

   bool needs_lowering(ir_dereference_array *deref) const
   {
      if (deref == NULL || deref->array_index->as_constant() ||
          !is_array_or_matrix(deref->array))
         return false;

      return this->storage_type_needs_lowering(deref);
   }

   ir_variable *convert_dereference_array(ir_dereference_array *orig_deref,
                                          ir_assignment* orig_assign,
                                          ir_dereference *orig_base)
   {
      void *const mem_ctx = ralloc_parent(base_ir);
      exec_list list;
      ir_factory body(&list, mem_ctx);

      assert(is_array_or_matrix(orig_deref->array));

      const unsigned length = (orig_deref->array->type->is_array())
         ? orig_deref->array->type->length
         : orig_deref->array->type->matrix_columns;

      /* Temporary storage for either the result of the dereference of
       * the array, or the RHS that's being assigned into the
       * dereference of the array.
       */
      ir_variable *var;

      if (orig_assign) {
         var = body.make_temp(orig_assign->rhs->type,
                              "dereference_array_value");

         body.emit(assign(var, orig_assign->rhs));
      } else {
         var = body.make_temp(orig_deref->type,
                              "dereference_array_value");
      }

      /* Store the index to a temporary to avoid reusing its tree. */
      ir_variable *index = body.make_temp(orig_deref->array_index->type,
                                          "dereference_array_index");

      body.emit(assign(index, orig_deref->array_index));

      orig_deref->array_index = deref(index).val;

      assignment_generator ag;
      ag.rvalue = orig_base;
      ag.base_ir = base_ir;
      ag.old_index = index;
      ag.var = var;
      if (orig_assign) {
         ag.is_write = true;
         ag.write_mask = orig_assign->write_mask;
      } else {
         ag.is_write = false;
      }

      switch_generator sg(ag, index, 4, 4);

      /* If the original assignment has a condition, respect that original
       * condition!  This is acomplished by wrapping the new conditional
       * assignments in an if-statement that uses the original condition.
       */
      if (orig_assign != NULL && orig_assign->condition != NULL) {
         /* No need to clone the condition because the IR that it hangs on is
          * going to be removed from the instruction sequence.
          */
         ir_if *if_stmt = new(mem_ctx) ir_if(orig_assign->condition);
         ir_factory then_body(&if_stmt->then_instructions, body.mem_ctx);

         sg.generate(0, length, then_body);
         body.emit(if_stmt);
      } else {
         sg.generate(0, length, body);
      }

      base_ir->insert_before(&list);
      return var;
   }

   virtual void handle_rvalue(ir_rvalue **pir)
   {
      if (this->in_assignee)
         return;

      if (!*pir)
         return;

      ir_dereference_array* orig_deref = (*pir)->as_dereference_array();
      if (needs_lowering(orig_deref)) {
         ir_variable *var =
            convert_dereference_array(orig_deref, NULL, orig_deref);
         assert(var);
         *pir = new(ralloc_parent(base_ir)) ir_dereference_variable(var);
         this->progress = true;
      }
   }

   ir_visitor_status
   visit_leave(ir_assignment *ir)
   {
      ir_rvalue_visitor::visit_leave(ir);

      find_variable_index f;
      ir->lhs->accept(&f);

      if (f.deref != NULL && storage_type_needs_lowering(f.deref)) {
         convert_dereference_array(f.deref, ir, ir->lhs);
         ir->remove();
         this->progress = true;
      }

      return visit_continue;
   }
};

} /* anonymous namespace */

bool
lower_variable_index_to_cond_assign(gl_shader_stage stage,
                                    exec_list *instructions,
                                    bool lower_input,
                                    bool lower_output,
                                    bool lower_temp,
                                    bool lower_uniform)
{
   variable_index_to_cond_assign_visitor v(stage,
                                           lower_input,
                                           lower_output,
                                           lower_temp,
                                           lower_uniform);

   /* Continue lowering until no progress is made.  If there are multiple
    * levels of indirection (e.g., non-constant indexing of array elements and
    * matrix columns of an array of matrix), each pass will only lower one
    * level of indirection.
    */
   bool progress_ever = false;
   do {
      v.progress = false;
      visit_list_elements(&v, instructions);
      progress_ever = v.progress || progress_ever;
   } while (v.progress);

   return progress_ever;
}
