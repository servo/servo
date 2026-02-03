/*
 * Copyright © 2013 Marek Olšák <maraeo@gmail.com>
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
 * \file opt_dead_builtin_varyings.cpp
 *
 * This eliminates the built-in shader outputs which are either not written
 * at all or not used by the next stage. It also eliminates unused elements
 * of gl_TexCoord inputs, which reduces the overall varying usage.
 * The varyings handled here are the primary and secondary color, the fog,
 * and the texture coordinates (gl_TexCoord).
 *
 * This pass is necessary, because the Mesa GLSL linker cannot eliminate
 * built-in varyings like it eliminates user-defined varyings, because
 * the built-in varyings have pre-assigned locations. Also, the elimination
 * of unused gl_TexCoord elements requires its own lowering pass anyway.
 *
 * It's implemented by replacing all occurrences of dead varyings with
 * temporary variables, which creates dead code. It is recommended to run
 * a dead-code elimination pass after this.
 *
 * If any texture coordinate slots can be eliminated, the gl_TexCoord array is
 * broken down into separate vec4 variables with locations equal to
 * VARYING_SLOT_TEX0 + i.
 *
 * The same is done for the gl_FragData fragment shader output.
 */

#include "ir.h"
#include "ir_rvalue_visitor.h"
#include "ir_optimization.h"
#include "ir_print_visitor.h"
#include "compiler/glsl_types.h"
#include "link_varyings.h"
#include "main/mtypes.h"
#include "util/u_string.h"

namespace {

/**
 * This obtains detailed information about built-in varyings from shader code.
 */
class varying_info_visitor : public ir_hierarchical_visitor {
public:
   /* "mode" can be either ir_var_shader_in or ir_var_shader_out */
   varying_info_visitor(ir_variable_mode mode, bool find_frag_outputs = false)
      : lower_texcoord_array(true),
        texcoord_array(NULL),
        texcoord_usage(0),
        find_frag_outputs(find_frag_outputs),
        lower_fragdata_array(true),
        fragdata_array(NULL),
        fragdata_usage(0),
        color_usage(0),
        tfeedback_color_usage(0),
        fog(NULL),
        has_fog(false),
        tfeedback_has_fog(false),
        mode(mode)
   {
      memset(color, 0, sizeof(color));
      memset(backcolor, 0, sizeof(backcolor));
   }

   virtual ir_visitor_status visit_enter(ir_dereference_array *ir)
   {
      ir_variable *var = ir->variable_referenced();

      if (!var || var->data.mode != this->mode || !var->type->is_array() ||
          !is_gl_identifier(var->name))
         return visit_continue;

      /* Only match gl_FragData[], not gl_SecondaryFragDataEXT[] or
       * gl_LastFragData[].
       */
      if (this->find_frag_outputs && strcmp(var->name, "gl_FragData") == 0) {
         this->fragdata_array = var;

         ir_constant *index = ir->array_index->as_constant();
         if (index == NULL) {
            /* This is variable indexing. */
            this->fragdata_usage |= (1 << var->type->array_size()) - 1;
            this->lower_fragdata_array = false;
         }
         else {
            this->fragdata_usage |= 1 << index->get_uint_component(0);
            /* Don't lower fragdata array if the output variable
             * is not a float variable (or float vector) because it will
             * generate wrong register assignments because of different
             * data types.
             */
            if (var->type->gl_type != GL_FLOAT &&
                var->type->gl_type != GL_FLOAT_VEC2 &&
                var->type->gl_type != GL_FLOAT_VEC3 &&
                var->type->gl_type != GL_FLOAT_VEC4)
               this->lower_fragdata_array = false;
         }

         /* Don't visit the leaves of ir_dereference_array. */
         return visit_continue_with_parent;
      }

      if (!this->find_frag_outputs && var->data.location == VARYING_SLOT_TEX0) {
         this->texcoord_array = var;

         ir_constant *index = ir->array_index->as_constant();
         if (index == NULL) {
            /* There is variable indexing, we can't lower the texcoord array.
             */
            this->texcoord_usage |= (1 << var->type->array_size()) - 1;
            this->lower_texcoord_array = false;
         }
         else {
            this->texcoord_usage |= 1 << index->get_uint_component(0);
         }

         /* Don't visit the leaves of ir_dereference_array. */
         return visit_continue_with_parent;
      }

      return visit_continue;
   }

   virtual ir_visitor_status visit(ir_dereference_variable *ir)
   {
      ir_variable *var = ir->variable_referenced();

      if (var->data.mode != this->mode || !var->type->is_array())
         return visit_continue;

      if (this->find_frag_outputs && var->data.location == FRAG_RESULT_DATA0 &&
          var->data.index == 0) {
         /* This is a whole array dereference. */
         this->fragdata_usage |= (1 << var->type->array_size()) - 1;
         this->lower_fragdata_array = false;
         return visit_continue;
      }

      if (!this->find_frag_outputs && var->data.location == VARYING_SLOT_TEX0) {
         /* This is a whole array dereference like "gl_TexCoord = x;",
          * there's probably no point in lowering that.
          */
         this->texcoord_usage |= (1 << var->type->array_size()) - 1;
         this->lower_texcoord_array = false;
      }
      return visit_continue;
   }

   virtual ir_visitor_status visit(ir_variable *var)
   {
      if (var->data.mode != this->mode)
         return visit_continue;

      /* Nothing to do here for fragment outputs. */
      if (this->find_frag_outputs)
         return visit_continue;

      /* Handle colors and fog. */
      switch (var->data.location) {
      case VARYING_SLOT_COL0:
         this->color[0] = var;
         this->color_usage |= 1;
         break;
      case VARYING_SLOT_COL1:
         this->color[1] = var;
         this->color_usage |= 2;
         break;
      case VARYING_SLOT_BFC0:
         this->backcolor[0] = var;
         this->color_usage |= 1;
         break;
      case VARYING_SLOT_BFC1:
         this->backcolor[1] = var;
         this->color_usage |= 2;
         break;
      case VARYING_SLOT_FOGC:
         this->fog = var;
         this->has_fog = true;
         break;
      }

      return visit_continue;
   }

   void get(exec_list *ir,
            unsigned num_tfeedback_decls,
            tfeedback_decl *tfeedback_decls)
   {
      /* Handle the transform feedback varyings. */
      for (unsigned i = 0; i < num_tfeedback_decls; i++) {
         if (!tfeedback_decls[i].is_varying())
            continue;

         unsigned location = tfeedback_decls[i].get_location();

         switch (location) {
         case VARYING_SLOT_COL0:
         case VARYING_SLOT_BFC0:
            this->tfeedback_color_usage |= 1;
            break;
         case VARYING_SLOT_COL1:
         case VARYING_SLOT_BFC1:
            this->tfeedback_color_usage |= 2;
            break;
         case VARYING_SLOT_FOGC:
            this->tfeedback_has_fog = true;
            break;
         default:
            if (location >= VARYING_SLOT_TEX0 &&
                location <= VARYING_SLOT_TEX7) {
               this->lower_texcoord_array = false;
            }
         }
      }

      /* Process the shader. */
      visit_list_elements(this, ir);

      if (!this->texcoord_array) {
         this->lower_texcoord_array = false;
      }
      if (!this->fragdata_array) {
         this->lower_fragdata_array = false;
      }
   }

   bool lower_texcoord_array;
   ir_variable *texcoord_array;
   unsigned texcoord_usage; /* bitmask */

   bool find_frag_outputs; /* false if it's looking for varyings */
   bool lower_fragdata_array;
   ir_variable *fragdata_array;
   unsigned fragdata_usage; /* bitmask */

   ir_variable *color[2];
   ir_variable *backcolor[2];
   unsigned color_usage; /* bitmask */
   unsigned tfeedback_color_usage; /* bitmask */

   ir_variable *fog;
   bool has_fog;
   bool tfeedback_has_fog;

   ir_variable_mode mode;
};


/**
 * This replaces unused varyings with temporary variables.
 *
 * If "ir" is the producer, the "external" usage should come from
 * the consumer. It also works the other way around. If either one is
 * missing, set the "external" usage to a full mask.
 */
class replace_varyings_visitor : public ir_rvalue_visitor {
public:
   replace_varyings_visitor(struct gl_linked_shader *sha,
                            const varying_info_visitor *info,
                            unsigned external_texcoord_usage,
                            unsigned external_color_usage,
                            bool external_has_fog)
      : shader(sha), info(info), new_fog(NULL)
   {
      void *const ctx = shader->ir;

      memset(this->new_fragdata, 0, sizeof(this->new_fragdata));
      memset(this->new_texcoord, 0, sizeof(this->new_texcoord));
      memset(this->new_color, 0, sizeof(this->new_color));
      memset(this->new_backcolor, 0, sizeof(this->new_backcolor));

      const char *mode_str =
         info->mode == ir_var_shader_in ? "in" : "out";

      /* Handle texcoord outputs.
       *
       * We're going to break down the gl_TexCoord array into separate
       * variables. First, add declarations of the new variables all
       * occurrences of gl_TexCoord will be replaced with.
       */
      if (info->lower_texcoord_array) {
         prepare_array(shader->ir, this->new_texcoord,
                       ARRAY_SIZE(this->new_texcoord),
                       VARYING_SLOT_TEX0, "TexCoord", mode_str,
                       info->texcoord_usage, external_texcoord_usage);
      }

      /* Handle gl_FragData in the same way like gl_TexCoord. */
      if (info->lower_fragdata_array) {
         prepare_array(shader->ir, this->new_fragdata,
                       ARRAY_SIZE(this->new_fragdata),
                       FRAG_RESULT_DATA0, "FragData", mode_str,
                       info->fragdata_usage, (1 << MAX_DRAW_BUFFERS) - 1);
      }

      /* Create dummy variables which will replace set-but-unused color and
       * fog outputs.
       */
      external_color_usage |= info->tfeedback_color_usage;

      for (int i = 0; i < 2; i++) {
         char name[32];

         if (!(external_color_usage & (1 << i))) {
            if (info->color[i]) {
               snprintf(name, 32, "gl_%s_FrontColor%i_dummy", mode_str, i);
               this->new_color[i] =
                  new (ctx) ir_variable(glsl_type::vec4_type, name,
                                        ir_var_temporary);
            }

            if (info->backcolor[i]) {
               snprintf(name, 32, "gl_%s_BackColor%i_dummy", mode_str, i);
               this->new_backcolor[i] =
                  new (ctx) ir_variable(glsl_type::vec4_type, name,
                                        ir_var_temporary);
            }
         }
      }

      if (!external_has_fog && !info->tfeedback_has_fog &&
          info->fog) {
         char name[32];

         snprintf(name, 32, "gl_%s_FogFragCoord_dummy", mode_str);
         this->new_fog = new (ctx) ir_variable(glsl_type::float_type, name,
                                               ir_var_temporary);
      }

      /* Now do the replacing. */
      visit_list_elements(this, shader->ir);
   }

   void prepare_array(exec_list *ir,
                      ir_variable **new_var,
                      int max_elements, unsigned start_location,
                      const char *var_name, const char *mode_str,
                      unsigned usage, unsigned external_usage)
   {
      void *const ctx = ir;

      for (int i = max_elements-1; i >= 0; i--) {
         if (usage & (1 << i)) {
            char name[32];

            if (!(external_usage & (1 << i))) {
               /* This varying is unused in the next stage. Declare
                * a temporary instead of an output. */
               snprintf(name, 32, "gl_%s_%s%i_dummy", mode_str, var_name, i);
               new_var[i] =
                  new (ctx) ir_variable(glsl_type::vec4_type, name,
                                        ir_var_temporary);
            }
            else {
               snprintf(name, 32, "gl_%s_%s%i", mode_str, var_name, i);
               new_var[i] =
                  new(ctx) ir_variable(glsl_type::vec4_type, name,
                                       this->info->mode);
               new_var[i]->data.location = start_location + i;
               new_var[i]->data.explicit_location = true;
               new_var[i]->data.explicit_index = 0;
            }

            ir->get_head_raw()->insert_before(new_var[i]);
         }
      }
   }

   virtual ir_visitor_status visit(ir_variable *var)
   {
      /* Remove the gl_TexCoord array. */
      if (this->info->lower_texcoord_array &&
          var == this->info->texcoord_array) {
         var->remove();
      }

      /* Remove the gl_FragData array. */
      if (this->info->lower_fragdata_array &&
          var == this->info->fragdata_array) {

         /* Clone variable for program resource list before it is removed. */
         if (!shader->fragdata_arrays)
            shader->fragdata_arrays = new (shader) exec_list;

         shader->fragdata_arrays->push_tail(var->clone(shader, NULL));

         var->remove();
      }

      /* Replace set-but-unused color and fog outputs with dummy variables. */
      for (int i = 0; i < 2; i++) {
         if (var == this->info->color[i] && this->new_color[i]) {
            var->replace_with(this->new_color[i]);
         }
         if (var == this->info->backcolor[i] &&
             this->new_backcolor[i]) {
            var->replace_with(this->new_backcolor[i]);
         }
      }

      if (var == this->info->fog && this->new_fog) {
         var->replace_with(this->new_fog);
      }

      return visit_continue;
   }

   virtual void handle_rvalue(ir_rvalue **rvalue)
   {
      if (!*rvalue)
         return;

      void *ctx = ralloc_parent(*rvalue);

      /* Replace an array dereference gl_TexCoord[i] with a single
       * variable dereference representing gl_TexCoord[i].
       */
      if (this->info->lower_texcoord_array) {
         /* gl_TexCoord[i] occurrence */
         ir_dereference_array *const da = (*rvalue)->as_dereference_array();

         if (da && da->variable_referenced() ==
             this->info->texcoord_array) {
            unsigned i = da->array_index->as_constant()->get_uint_component(0);

            *rvalue = new(ctx) ir_dereference_variable(this->new_texcoord[i]);
            return;
         }
      }

      /* Same for gl_FragData. */
      if (this->info->lower_fragdata_array) {
         /* gl_FragData[i] occurrence */
         ir_dereference_array *const da = (*rvalue)->as_dereference_array();

         if (da && da->variable_referenced() == this->info->fragdata_array) {
            unsigned i = da->array_index->as_constant()->get_uint_component(0);

            *rvalue = new(ctx) ir_dereference_variable(this->new_fragdata[i]);
            return;
         }
      }

      /* Replace set-but-unused color and fog outputs with dummy variables. */
      ir_dereference_variable *const dv = (*rvalue)->as_dereference_variable();
      if (!dv)
         return;

      ir_variable *var = dv->variable_referenced();

      for (int i = 0; i < 2; i++) {
         if (var == this->info->color[i] && this->new_color[i]) {
            *rvalue = new(ctx) ir_dereference_variable(this->new_color[i]);
            return;
         }
         if (var == this->info->backcolor[i] &&
             this->new_backcolor[i]) {
            *rvalue = new(ctx) ir_dereference_variable(this->new_backcolor[i]);
            return;
         }
      }

      if (var == this->info->fog && this->new_fog) {
         *rvalue = new(ctx) ir_dereference_variable(this->new_fog);
      }
   }

   virtual ir_visitor_status visit_leave(ir_assignment *ir)
   {
      handle_rvalue(&ir->rhs);
      handle_rvalue(&ir->condition);

      /* We have to use set_lhs when changing the LHS of an assignment. */
      ir_rvalue *lhs = ir->lhs;

      handle_rvalue(&lhs);
      if (lhs != ir->lhs) {
         ir->set_lhs(lhs);
      }

      return visit_continue;
   }

private:
   struct gl_linked_shader *shader;
   const varying_info_visitor *info;
   ir_variable *new_fragdata[MAX_DRAW_BUFFERS];
   ir_variable *new_texcoord[MAX_TEXTURE_COORD_UNITS];
   ir_variable *new_color[2];
   ir_variable *new_backcolor[2];
   ir_variable *new_fog;
};

} /* anonymous namespace */

static void
lower_texcoord_array(struct gl_linked_shader *shader, const varying_info_visitor *info)
{
   replace_varyings_visitor(shader, info,
                            (1 << MAX_TEXTURE_COORD_UNITS) - 1,
                            1 | 2, true);
}

static void
lower_fragdata_array(struct gl_linked_shader *shader)
{
   varying_info_visitor info(ir_var_shader_out, true);
   info.get(shader->ir, 0, NULL);

   replace_varyings_visitor(shader, &info, 0, 0, 0);
}


void
do_dead_builtin_varyings(struct gl_context *ctx,
                         gl_linked_shader *producer,
                         gl_linked_shader *consumer,
                         unsigned num_tfeedback_decls,
                         tfeedback_decl *tfeedback_decls)
{
   /* Lower the gl_FragData array to separate variables. */
   if (consumer && consumer->Stage == MESA_SHADER_FRAGMENT &&
       !ctx->Const.ShaderCompilerOptions[MESA_SHADER_FRAGMENT].NirOptions) {
      lower_fragdata_array(consumer);
   }

   /* Lowering of built-in varyings has no effect with the core context and
    * GLES2, because they are not available there.
    */
   if (ctx->API == API_OPENGL_CORE ||
       ctx->API == API_OPENGLES2) {
      return;
   }

   /* Information about built-in varyings. */
   varying_info_visitor producer_info(ir_var_shader_out);
   varying_info_visitor consumer_info(ir_var_shader_in);

   if (producer) {
      producer_info.get(producer->ir, num_tfeedback_decls, tfeedback_decls);

      if (producer->Stage == MESA_SHADER_TESS_CTRL)
         producer_info.lower_texcoord_array = false;

      if (!consumer) {
         /* At least eliminate unused gl_TexCoord elements. */
         if (producer_info.lower_texcoord_array) {
            lower_texcoord_array(producer, &producer_info);
         }
         return;
      }
   }

   if (consumer) {
      consumer_info.get(consumer->ir, 0, NULL);

      if (consumer->Stage != MESA_SHADER_FRAGMENT)
         consumer_info.lower_texcoord_array = false;

      if (!producer) {
         /* At least eliminate unused gl_TexCoord elements. */
         if (consumer_info.lower_texcoord_array) {
            lower_texcoord_array(consumer, &consumer_info);
         }
         return;
      }
   }

   /* Eliminate the outputs unused by the consumer. */
   if (producer_info.lower_texcoord_array ||
       producer_info.color_usage ||
       producer_info.has_fog) {
      replace_varyings_visitor(producer,
                               &producer_info,
                               consumer_info.texcoord_usage,
                               consumer_info.color_usage,
                               consumer_info.has_fog);
   }

   /* The gl_TexCoord fragment shader inputs can be initialized
    * by GL_COORD_REPLACE, so we can't eliminate them.
    *
    * This doesn't prevent elimination of the gl_TexCoord elements which
    * are not read by the fragment shader. We want to eliminate those anyway.
    */
   if (consumer->Stage == MESA_SHADER_FRAGMENT) {
      producer_info.texcoord_usage = (1 << MAX_TEXTURE_COORD_UNITS) - 1;
   }

   /* Eliminate the inputs uninitialized by the producer. */
   if (consumer_info.lower_texcoord_array ||
       consumer_info.color_usage ||
       consumer_info.has_fog) {
      replace_varyings_visitor(consumer,
                               &consumer_info,
                               producer_info.texcoord_usage,
                               producer_info.color_usage,
                               producer_info.has_fog);
   }
}
