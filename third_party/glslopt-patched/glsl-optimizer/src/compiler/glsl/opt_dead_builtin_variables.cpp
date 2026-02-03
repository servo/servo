/*
 * Copyright © 2014 Intel Corporation
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
#include "ir_visitor.h"
#include "ir_optimization.h"

/**
 * Pre-linking, optimize unused built-in variables
 *
 * Uniforms, constants, system values, inputs (vertex shader only), and
 * outputs (fragment shader only) that are not used can be removed.
 */
void
optimize_dead_builtin_variables(exec_list *instructions,
                                enum ir_variable_mode other)
{
   foreach_in_list_safe(ir_variable, var, instructions) {
      if (var->ir_type != ir_type_variable || var->data.used)
         continue;

      if (var->data.mode != ir_var_uniform
          && var->data.mode != ir_var_auto
          && var->data.mode != ir_var_system_value
          && var->data.mode != other)
         continue;

      /* So that linker rules can later be enforced, we cannot elimate
       * variables that were redeclared in the shader code.
       */
      if ((var->data.mode == other || var->data.mode == ir_var_system_value)
          && var->data.how_declared != ir_var_declared_implicitly)
         continue;

      if (!is_gl_identifier(var->name))
         continue;

      /* gl_ModelViewProjectionMatrix and gl_Vertex are special because they
       * are used by ftransform.  No other built-in variable is used by a
       * built-in function.  The forward declarations of these variables in
       * the built-in function shader does not have the "state slot"
       * information, so removing these variables from the user shader will
       * cause problems later.
       *
       * Matrix uniforms with "Transpose" are not eliminated because there's
       * an optimization pass that can turn references to the regular matrix
       * into references to the transpose matrix.  Eliminating the transpose
       * matrix would cause that pass to generate references to undeclareds
       * variables (thank you, ir_validate).
       *
       * It doesn't seem worth the effort to track when the transpose could be
       * eliminated (i.e., when the non-transpose was eliminated).
       */
      if (strcmp(var->name, "gl_ModelViewProjectionMatrix") == 0
          || strcmp(var->name, "gl_Vertex") == 0
          || strstr(var->name, "Transpose") != NULL)
         continue;

      var->remove();
   }
}
