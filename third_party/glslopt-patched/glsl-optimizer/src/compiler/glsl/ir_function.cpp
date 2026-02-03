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

#include "compiler/glsl_types.h"
#include "ir.h"
#include "glsl_parser_extras.h"
#include "main/errors.h"

typedef enum {
   PARAMETER_LIST_NO_MATCH,
   PARAMETER_LIST_EXACT_MATCH,
   PARAMETER_LIST_INEXACT_MATCH /*< Match requires implicit conversion. */
} parameter_list_match_t;

/**
 * \brief Check if two parameter lists match.
 *
 * \param list_a Parameters of the function definition.
 * \param list_b Actual parameters passed to the function.
 * \see matching_signature()
 */
static parameter_list_match_t
parameter_lists_match(_mesa_glsl_parse_state *state,
                      const exec_list *list_a, const exec_list *list_b)
{
   const exec_node *node_a = list_a->get_head_raw();
   const exec_node *node_b = list_b->get_head_raw();

   /* This is set to true if there is an inexact match requiring an implicit
    * conversion. */
   bool inexact_match = false;

   for (/* empty */
	; !node_a->is_tail_sentinel()
	; node_a = node_a->next, node_b = node_b->next) {
      /* If all of the parameters from the other parameter list have been
       * exhausted, the lists have different length and, by definition,
       * do not match.
       */
      if (node_b->is_tail_sentinel())
	 return PARAMETER_LIST_NO_MATCH;


      const ir_variable *const param = (ir_variable *) node_a;
      const ir_rvalue *const actual = (ir_rvalue *) node_b;

      if (param->type == actual->type)
	 continue;

      /* Try to find an implicit conversion from actual to param. */
      inexact_match = true;
      switch ((enum ir_variable_mode)(param->data.mode)) {
      case ir_var_auto:
      case ir_var_uniform:
      case ir_var_shader_storage:
      case ir_var_temporary:
	 /* These are all error conditions.  It is invalid for a parameter to
	  * a function to be declared as auto (not in, out, or inout) or
	  * as uniform.
	  */
	 assert(0);
	 return PARAMETER_LIST_NO_MATCH;

      case ir_var_const_in:
      case ir_var_function_in:
	 if (!actual->type->can_implicitly_convert_to(param->type, state))
	    return PARAMETER_LIST_NO_MATCH;
	 break;

      case ir_var_function_out:
	 if (!param->type->can_implicitly_convert_to(actual->type, state))
	    return PARAMETER_LIST_NO_MATCH;
	 break;

      case ir_var_function_inout:
	 /* Since there are no bi-directional automatic conversions (e.g.,
	  * there is int -> float but no float -> int), inout parameters must
	  * be exact matches.
	  */
	 return PARAMETER_LIST_NO_MATCH;

      default:
	 assert(false);
	 return PARAMETER_LIST_NO_MATCH;
      }
   }

   /* If all of the parameters from the other parameter list have been
    * exhausted, the lists have different length and, by definition, do not
    * match.
    */
   if (!node_b->is_tail_sentinel())
      return PARAMETER_LIST_NO_MATCH;

   if (inexact_match)
      return PARAMETER_LIST_INEXACT_MATCH;
   else
      return PARAMETER_LIST_EXACT_MATCH;
}


/* Classes of parameter match, sorted (mostly) best matches first.
 * See is_better_parameter_match() below for the exceptions.
 * */
typedef enum {
   PARAMETER_EXACT_MATCH,
   PARAMETER_FLOAT_TO_DOUBLE,
   PARAMETER_INT_TO_FLOAT,
   PARAMETER_INT_TO_DOUBLE,
   PARAMETER_OTHER_CONVERSION,
} parameter_match_t;


static parameter_match_t
get_parameter_match_type(const ir_variable *param,
                         const ir_rvalue *actual)
{
   const glsl_type *from_type;
   const glsl_type *to_type;

   if (param->data.mode == ir_var_function_out) {
      from_type = param->type;
      to_type = actual->type;
   } else {
      from_type = actual->type;
      to_type = param->type;
   }

   if (from_type == to_type)
      return PARAMETER_EXACT_MATCH;

   if (to_type->is_double()) {
      if (from_type->is_float())
         return PARAMETER_FLOAT_TO_DOUBLE;
      return PARAMETER_INT_TO_DOUBLE;
   }

   if (to_type->is_float())
      return PARAMETER_INT_TO_FLOAT;

   /* int -> uint and any other oddball conversions */
   return PARAMETER_OTHER_CONVERSION;
}


static bool
is_better_parameter_match(parameter_match_t a_match,
                          parameter_match_t b_match)
{
   /* From section 6.1 of the GLSL 4.00 spec (and the ARB_gpu_shader5 spec):
    *
    * 1. An exact match is better than a match involving any implicit
    * conversion.
    *
    * 2. A match involving an implicit conversion from float to double
    * is better than match involving any other implicit conversion.
    *
    * [XXX: Not in GLSL 4.0: Only in ARB_gpu_shader5:
    * 3. A match involving an implicit conversion from either int or uint
    * to float is better than a match involving an implicit conversion
    * from either int or uint to double.]
    *
    * If none of the rules above apply to a particular pair of conversions,
    * neither conversion is considered better than the other.
    *
    * --
    *
    * Notably, the int->uint conversion is *not* considered to be better
    * or worse than int/uint->float or int/uint->double.
    */

   if (a_match >= PARAMETER_INT_TO_FLOAT && b_match == PARAMETER_OTHER_CONVERSION)
      return false;

   return a_match < b_match;
}


static bool
is_best_inexact_overload(const exec_list *actual_parameters,
                         ir_function_signature **matches,
                         int num_matches,
                         ir_function_signature *sig)
{
   /* From section 6.1 of the GLSL 4.00 spec (and the ARB_gpu_shader5 spec):
    *
    * "A function definition A is considered a better
    * match than function definition B if:
    *
    *   * for at least one function argument, the conversion for that argument
    *     in A is better than the corresponding conversion in B; and
    *
    *   * there is no function argument for which the conversion in B is better
    *     than the corresponding conversion in A.
    *
    * If a single function definition is considered a better match than every
    * other matching function definition, it will be used.  Otherwise, a
    * semantic error occurs and the shader will fail to compile."
    */
   for (ir_function_signature **other = matches;
        other < matches + num_matches; other++) {
      if (*other == sig)
         continue;

      const exec_node *node_a = sig->parameters.get_head_raw();
      const exec_node *node_b = (*other)->parameters.get_head_raw();
      const exec_node *node_p = actual_parameters->get_head_raw();

      bool better_for_some_parameter = false;

      for (/* empty */
           ; !node_a->is_tail_sentinel()
           ; node_a = node_a->next,
             node_b = node_b->next,
             node_p = node_p->next) {
         parameter_match_t a_match = get_parameter_match_type(
               (const ir_variable *)node_a,
               (const ir_rvalue *)node_p);
         parameter_match_t b_match = get_parameter_match_type(
               (const ir_variable *)node_b,
               (const ir_rvalue *)node_p);

         if (is_better_parameter_match(a_match, b_match))
               better_for_some_parameter = true;

         if (is_better_parameter_match(b_match, a_match))
               return false;     /* B is better for this parameter */
      }

      if (!better_for_some_parameter)
         return false;     /* A must be better than B for some parameter */

   }

   return true;
}


static ir_function_signature *
choose_best_inexact_overload(_mesa_glsl_parse_state *state,
                             const exec_list *actual_parameters,
                             ir_function_signature **matches,
                             int num_matches)
{
   if (num_matches == 0)
      return NULL;

   if (num_matches == 1)
      return *matches;

   /* Without GLSL 4.0, ARB_gpu_shader5, or MESA_shader_integer_functions,
    * there is no overload resolution among multiple inexact matches. Note
    * that state may be NULL here if called from the linker; in that case we
    * assume everything supported in any GLSL version is available.
    */
   if (!state || state->is_version(400, 0) || state->ARB_gpu_shader5_enable ||
       state->MESA_shader_integer_functions_enable ||
       state->EXT_shader_implicit_conversions_enable) {
      for (ir_function_signature **sig = matches; sig < matches + num_matches; sig++) {
         if (is_best_inexact_overload(actual_parameters, matches, num_matches, *sig))
            return *sig;
      }
   }

   return NULL;   /* no best candidate */
}


ir_function_signature *
ir_function::matching_signature(_mesa_glsl_parse_state *state,
                                const exec_list *actual_parameters,
                                bool allow_builtins)
{
   bool is_exact;
   return matching_signature(state, actual_parameters, allow_builtins,
                             &is_exact);
}

ir_function_signature *
ir_function::matching_signature(_mesa_glsl_parse_state *state,
                                const exec_list *actual_parameters,
                                bool allow_builtins,
                                bool *is_exact)
{
   ir_function_signature **inexact_matches = NULL;
   ir_function_signature **inexact_matches_temp;
   ir_function_signature *match = NULL;
   int num_inexact_matches = 0;

   /* From page 42 (page 49 of the PDF) of the GLSL 1.20 spec:
    *
    * "If an exact match is found, the other signatures are ignored, and
    *  the exact match is used.  Otherwise, if no exact match is found, then
    *  the implicit conversions in Section 4.1.10 "Implicit Conversions" will
    *  be applied to the calling arguments if this can make their types match
    *  a signature.  In this case, it is a semantic error if there are
    *  multiple ways to apply these conversions to the actual arguments of a
    *  call such that the call can be made to match multiple signatures."
    */
   foreach_in_list(ir_function_signature, sig, &this->signatures) {
      /* Skip over any built-ins that aren't available in this shader. */
      if (sig->is_builtin() && (!allow_builtins ||
                                !sig->is_builtin_available(state)))
         continue;

      switch (parameter_lists_match(state, & sig->parameters, actual_parameters)) {
      case PARAMETER_LIST_EXACT_MATCH:
         *is_exact = true;
         free(inexact_matches);
         return sig;
      case PARAMETER_LIST_INEXACT_MATCH:
         inexact_matches_temp = (ir_function_signature **)
               realloc(inexact_matches,
                       sizeof(*inexact_matches) *
                       (num_inexact_matches + 1));
         if (inexact_matches_temp == NULL) {
            _mesa_error_no_memory(__func__);
            free(inexact_matches);
            return NULL;
         }
         inexact_matches = inexact_matches_temp;
         inexact_matches[num_inexact_matches++] = sig;
         continue;
      case PARAMETER_LIST_NO_MATCH:
	 continue;
      default:
	 assert(false);
	 return NULL;
      }
   }

   /* There is no exact match (we would have returned it by now).  If there
    * are multiple inexact matches, the call is ambiguous, which is an error.
    *
    * FINISHME: Report a decent error.  Returning NULL will likely result in
    * FINISHME: a "no matching signature" error; it should report that the
    * FINISHME: call is ambiguous.  But reporting errors from here is hard.
    */
   *is_exact = false;

   match = choose_best_inexact_overload(state, actual_parameters,
                                        inexact_matches, num_inexact_matches);

   free(inexact_matches);
   return match;
}


static bool
parameter_lists_match_exact(const exec_list *list_a, const exec_list *list_b)
{
   const exec_node *node_a = list_a->get_head_raw();
   const exec_node *node_b = list_b->get_head_raw();

   for (/* empty */
	; !node_a->is_tail_sentinel() && !node_b->is_tail_sentinel()
	; node_a = node_a->next, node_b = node_b->next) {
      ir_variable *a = (ir_variable *) node_a;
      ir_variable *b = (ir_variable *) node_b;

      /* If the types of the parameters do not match, the parameters lists
       * are different.
       */
      if (a->type != b->type)
         return false;
   }

   /* Unless both lists are exhausted, they differ in length and, by
    * definition, do not match.
    */
   return (node_a->is_tail_sentinel() == node_b->is_tail_sentinel());
}

ir_function_signature *
ir_function::exact_matching_signature(_mesa_glsl_parse_state *state,
                                      const exec_list *actual_parameters)
{
   foreach_in_list(ir_function_signature, sig, &this->signatures) {
      /* Skip over any built-ins that aren't available in this shader. */
      if (sig->is_builtin() && !sig->is_builtin_available(state))
         continue;

      if (parameter_lists_match_exact(&sig->parameters, actual_parameters))
	 return sig;
   }
   return NULL;
}
