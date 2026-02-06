/*
 * Copyright © 2012 Intel Corporation
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
 * \file link_varyings.cpp
 *
 * Linker functions related specifically to linking varyings between shader
 * stages.
 */


#include "main/errors.h"
#include "main/mtypes.h"
#include "glsl_symbol_table.h"
#include "glsl_parser_extras.h"
#include "ir_optimization.h"
#include "linker.h"
#include "link_varyings.h"
#include "main/macros.h"
#include "util/hash_table.h"
#include "util/u_math.h"
#include "program.h"


/**
 * Get the varying type stripped of the outermost array if we're processing
 * a stage whose varyings are arrays indexed by a vertex number (such as
 * geometry shader inputs).
 */
static const glsl_type *
get_varying_type(const ir_variable *var, gl_shader_stage stage)
{
   const glsl_type *type = var->type;

   if (!var->data.patch &&
       ((var->data.mode == ir_var_shader_out &&
         stage == MESA_SHADER_TESS_CTRL) ||
        (var->data.mode == ir_var_shader_in &&
         (stage == MESA_SHADER_TESS_CTRL || stage == MESA_SHADER_TESS_EVAL ||
          stage == MESA_SHADER_GEOMETRY)))) {
      assert(type->is_array());
      type = type->fields.array;
   }

   return type;
}

static void
create_xfb_varying_names(void *mem_ctx, const glsl_type *t, char **name,
                         size_t name_length, unsigned *count,
                         const char *ifc_member_name,
                         const glsl_type *ifc_member_t, char ***varying_names)
{
   if (t->is_interface()) {
      size_t new_length = name_length;

      assert(ifc_member_name && ifc_member_t);
      ralloc_asprintf_rewrite_tail(name, &new_length, ".%s", ifc_member_name);

      create_xfb_varying_names(mem_ctx, ifc_member_t, name, new_length, count,
                               NULL, NULL, varying_names);
   } else if (t->is_struct()) {
      for (unsigned i = 0; i < t->length; i++) {
         const char *field = t->fields.structure[i].name;
         size_t new_length = name_length;

         ralloc_asprintf_rewrite_tail(name, &new_length, ".%s", field);

         create_xfb_varying_names(mem_ctx, t->fields.structure[i].type, name,
                                  new_length, count, NULL, NULL,
                                  varying_names);
      }
   } else if (t->without_array()->is_struct() ||
              t->without_array()->is_interface() ||
              (t->is_array() && t->fields.array->is_array())) {
      for (unsigned i = 0; i < t->length; i++) {
         size_t new_length = name_length;

         /* Append the subscript to the current variable name */
         ralloc_asprintf_rewrite_tail(name, &new_length, "[%u]", i);

         create_xfb_varying_names(mem_ctx, t->fields.array, name, new_length,
                                  count, ifc_member_name, ifc_member_t,
                                  varying_names);
      }
   } else {
      (*varying_names)[(*count)++] = ralloc_strdup(mem_ctx, *name);
   }
}

static bool
process_xfb_layout_qualifiers(void *mem_ctx, const gl_linked_shader *sh,
                              struct gl_shader_program *prog,
                              unsigned *num_tfeedback_decls,
                              char ***varying_names)
{
   bool has_xfb_qualifiers = false;

   /* We still need to enable transform feedback mode even if xfb_stride is
    * only applied to a global out. Also we don't bother to propagate
    * xfb_stride to interface block members so this will catch that case also.
    */
   for (unsigned j = 0; j < MAX_FEEDBACK_BUFFERS; j++) {
      if (prog->TransformFeedback.BufferStride[j]) {
         has_xfb_qualifiers = true;
         break;
      }
   }

   foreach_in_list(ir_instruction, node, sh->ir) {
      ir_variable *var = node->as_variable();
      if (!var || var->data.mode != ir_var_shader_out)
         continue;

      /* From the ARB_enhanced_layouts spec:
       *
       *    "Any shader making any static use (after preprocessing) of any of
       *     these *xfb_* qualifiers will cause the shader to be in a
       *     transform feedback capturing mode and hence responsible for
       *     describing the transform feedback setup.  This mode will capture
       *     any output selected by *xfb_offset*, directly or indirectly, to
       *     a transform feedback buffer."
       */
      if (var->data.explicit_xfb_buffer || var->data.explicit_xfb_stride) {
         has_xfb_qualifiers = true;
      }

      if (var->data.explicit_xfb_offset) {
         *num_tfeedback_decls += var->type->varying_count();
         has_xfb_qualifiers = true;
      }
   }

   if (*num_tfeedback_decls == 0)
      return has_xfb_qualifiers;

   unsigned i = 0;
   *varying_names = ralloc_array(mem_ctx, char *, *num_tfeedback_decls);
   foreach_in_list(ir_instruction, node, sh->ir) {
      ir_variable *var = node->as_variable();
      if (!var || var->data.mode != ir_var_shader_out)
         continue;

      if (var->data.explicit_xfb_offset) {
         char *name;
         const glsl_type *type, *member_type;

         if (var->data.from_named_ifc_block) {
            type = var->get_interface_type();

            /* Find the member type before it was altered by lowering */
            const glsl_type *type_wa = type->without_array();
            member_type =
               type_wa->fields.structure[type_wa->field_index(var->name)].type;
            name = ralloc_strdup(NULL, type_wa->name);
         } else {
            type = var->type;
            member_type = NULL;
            name = ralloc_strdup(NULL, var->name);
         }
         create_xfb_varying_names(mem_ctx, type, &name, strlen(name), &i,
                                  var->name, member_type, varying_names);
         ralloc_free(name);
      }
   }

   assert(i == *num_tfeedback_decls);
   return has_xfb_qualifiers;
}

/**
 * Validate the types and qualifiers of an output from one stage against the
 * matching input to another stage.
 */
static void
cross_validate_types_and_qualifiers(struct gl_context *ctx,
                                    struct gl_shader_program *prog,
                                    const ir_variable *input,
                                    const ir_variable *output,
                                    gl_shader_stage consumer_stage,
                                    gl_shader_stage producer_stage)
{
   /* Check that the types match between stages.
    */
   const glsl_type *type_to_match = input->type;

   /* VS -> GS, VS -> TCS, VS -> TES, TES -> GS */
   const bool extra_array_level = (producer_stage == MESA_SHADER_VERTEX &&
                                   consumer_stage != MESA_SHADER_FRAGMENT) ||
                                  consumer_stage == MESA_SHADER_GEOMETRY;
   if (extra_array_level) {
      assert(type_to_match->is_array());
      type_to_match = type_to_match->fields.array;
   }

   if (type_to_match != output->type) {
      if (output->type->is_struct()) {
         /* Structures across shader stages can have different name
          * and considered to match in type if and only if structure
          * members match in name, type, qualification, and declaration
          * order. The precision doesn’t need to match.
          */
         if (!output->type->record_compare(type_to_match,
                                           false, /* match_name */
                                           true, /* match_locations */
                                           false /* match_precision */)) {
            linker_error(prog,
                  "%s shader output `%s' declared as struct `%s', "
                  "doesn't match in type with %s shader input "
                  "declared as struct `%s'\n",
                  _mesa_shader_stage_to_string(producer_stage),
                  output->name,
                  output->type->name,
                  _mesa_shader_stage_to_string(consumer_stage),
                  input->type->name);
         }
      } else if (!output->type->is_array() || !is_gl_identifier(output->name)) {
         /* There is a bit of a special case for gl_TexCoord.  This
          * built-in is unsized by default.  Applications that variable
          * access it must redeclare it with a size.  There is some
          * language in the GLSL spec that implies the fragment shader
          * and vertex shader do not have to agree on this size.  Other
          * driver behave this way, and one or two applications seem to
          * rely on it.
          *
          * Neither declaration needs to be modified here because the array
          * sizes are fixed later when update_array_sizes is called.
          *
          * From page 48 (page 54 of the PDF) of the GLSL 1.10 spec:
          *
          *     "Unlike user-defined varying variables, the built-in
          *     varying variables don't have a strict one-to-one
          *     correspondence between the vertex language and the
          *     fragment language."
          */
         linker_error(prog,
                      "%s shader output `%s' declared as type `%s', "
                      "but %s shader input declared as type `%s'\n",
                      _mesa_shader_stage_to_string(producer_stage),
                      output->name,
                      output->type->name,
                      _mesa_shader_stage_to_string(consumer_stage),
                      input->type->name);
         return;
      }
   }

   /* Check that all of the qualifiers match between stages.
    */

   /* According to the OpenGL and OpenGLES GLSL specs, the centroid qualifier
    * should match until OpenGL 4.3 and OpenGLES 3.1. The OpenGLES 3.0
    * conformance test suite does not verify that the qualifiers must match.
    * The deqp test suite expects the opposite (OpenGLES 3.1) behavior for
    * OpenGLES 3.0 drivers, so we relax the checking in all cases.
    */
   if (false /* always skip the centroid check */ &&
       prog->data->Version < (prog->IsES ? 310 : 430) &&
       input->data.centroid != output->data.centroid) {
      linker_error(prog,
                   "%s shader output `%s' %s centroid qualifier, "
                   "but %s shader input %s centroid qualifier\n",
                   _mesa_shader_stage_to_string(producer_stage),
                   output->name,
                   (output->data.centroid) ? "has" : "lacks",
                   _mesa_shader_stage_to_string(consumer_stage),
                   (input->data.centroid) ? "has" : "lacks");
      return;
   }

   if (input->data.sample != output->data.sample) {
      linker_error(prog,
                   "%s shader output `%s' %s sample qualifier, "
                   "but %s shader input %s sample qualifier\n",
                   _mesa_shader_stage_to_string(producer_stage),
                   output->name,
                   (output->data.sample) ? "has" : "lacks",
                   _mesa_shader_stage_to_string(consumer_stage),
                   (input->data.sample) ? "has" : "lacks");
      return;
   }

   if (input->data.patch != output->data.patch) {
      linker_error(prog,
                   "%s shader output `%s' %s patch qualifier, "
                   "but %s shader input %s patch qualifier\n",
                   _mesa_shader_stage_to_string(producer_stage),
                   output->name,
                   (output->data.patch) ? "has" : "lacks",
                   _mesa_shader_stage_to_string(consumer_stage),
                   (input->data.patch) ? "has" : "lacks");
      return;
   }

   /* The GLSL 4.30 and GLSL ES 3.00 specifications say:
    *
    *    "As only outputs need be declared with invariant, an output from
    *     one shader stage will still match an input of a subsequent stage
    *     without the input being declared as invariant."
    *
    * while GLSL 4.20 says:
    *
    *    "For variables leaving one shader and coming into another shader,
    *     the invariant keyword has to be used in both shaders, or a link
    *     error will result."
    *
    * and GLSL ES 1.00 section 4.6.4 "Invariance and Linking" says:
    *
    *    "The invariance of varyings that are declared in both the vertex
    *     and fragment shaders must match."
    */
   if (input->data.explicit_invariant != output->data.explicit_invariant &&
       prog->data->Version < (prog->IsES ? 300 : 430)) {
      linker_error(prog,
                   "%s shader output `%s' %s invariant qualifier, "
                   "but %s shader input %s invariant qualifier\n",
                   _mesa_shader_stage_to_string(producer_stage),
                   output->name,
                   (output->data.explicit_invariant) ? "has" : "lacks",
                   _mesa_shader_stage_to_string(consumer_stage),
                   (input->data.explicit_invariant) ? "has" : "lacks");
      return;
   }

   /* GLSL >= 4.40 removes text requiring interpolation qualifiers
    * to match cross stage, they must only match within the same stage.
    *
    * From page 84 (page 90 of the PDF) of the GLSL 4.40 spec:
    *
    *     "It is a link-time error if, within the same stage, the interpolation
    *     qualifiers of variables of the same name do not match.
    *
    * Section 4.3.9 (Interpolation) of the GLSL ES 3.00 spec says:
    *
    *    "When no interpolation qualifier is present, smooth interpolation
    *    is used."
    *
    * So we match variables where one is smooth and the other has no explicit
    * qualifier.
    */
   unsigned input_interpolation = input->data.interpolation;
   unsigned output_interpolation = output->data.interpolation;
   if (prog->IsES) {
      if (input_interpolation == INTERP_MODE_NONE)
         input_interpolation = INTERP_MODE_SMOOTH;
      if (output_interpolation == INTERP_MODE_NONE)
         output_interpolation = INTERP_MODE_SMOOTH;
   }
   if (input_interpolation != output_interpolation &&
       prog->data->Version < 440) {
      if (!ctx->Const.AllowGLSLCrossStageInterpolationMismatch) {
         linker_error(prog,
                      "%s shader output `%s' specifies %s "
                      "interpolation qualifier, "
                      "but %s shader input specifies %s "
                      "interpolation qualifier\n",
                      _mesa_shader_stage_to_string(producer_stage),
                      output->name,
                      interpolation_string(output->data.interpolation),
                      _mesa_shader_stage_to_string(consumer_stage),
                      interpolation_string(input->data.interpolation));
         return;
      } else {
         linker_warning(prog,
                        "%s shader output `%s' specifies %s "
                        "interpolation qualifier, "
                        "but %s shader input specifies %s "
                        "interpolation qualifier\n",
                        _mesa_shader_stage_to_string(producer_stage),
                        output->name,
                        interpolation_string(output->data.interpolation),
                        _mesa_shader_stage_to_string(consumer_stage),
                        interpolation_string(input->data.interpolation));
      }
   }
}

/**
 * Validate front and back color outputs against single color input
 */
static void
cross_validate_front_and_back_color(struct gl_context *ctx,
                                    struct gl_shader_program *prog,
                                    const ir_variable *input,
                                    const ir_variable *front_color,
                                    const ir_variable *back_color,
                                    gl_shader_stage consumer_stage,
                                    gl_shader_stage producer_stage)
{
   if (front_color != NULL && front_color->data.assigned)
      cross_validate_types_and_qualifiers(ctx, prog, input, front_color,
                                          consumer_stage, producer_stage);

   if (back_color != NULL && back_color->data.assigned)
      cross_validate_types_and_qualifiers(ctx, prog, input, back_color,
                                          consumer_stage, producer_stage);
}

static unsigned
compute_variable_location_slot(ir_variable *var, gl_shader_stage stage)
{
   unsigned location_start = VARYING_SLOT_VAR0;

   switch (stage) {
      case MESA_SHADER_VERTEX:
         if (var->data.mode == ir_var_shader_in)
            location_start = VERT_ATTRIB_GENERIC0;
         break;
      case MESA_SHADER_TESS_CTRL:
      case MESA_SHADER_TESS_EVAL:
         if (var->data.patch)
            location_start = VARYING_SLOT_PATCH0;
         break;
      case MESA_SHADER_FRAGMENT:
         if (var->data.mode == ir_var_shader_out)
            location_start = FRAG_RESULT_DATA0;
         break;
      default:
         break;
   }

   return var->data.location - location_start;
}

struct explicit_location_info {
   ir_variable *var;
   bool base_type_is_integer;
   unsigned base_type_bit_size;
   unsigned interpolation;
   bool centroid;
   bool sample;
   bool patch;
};

static bool
check_location_aliasing(struct explicit_location_info explicit_locations[][4],
                        ir_variable *var,
                        unsigned location,
                        unsigned component,
                        unsigned location_limit,
                        const glsl_type *type,
                        unsigned interpolation,
                        bool centroid,
                        bool sample,
                        bool patch,
                        gl_shader_program *prog,
                        gl_shader_stage stage)
{
   unsigned last_comp;
   unsigned base_type_bit_size;
   const glsl_type *type_without_array = type->without_array();
   const bool base_type_is_integer =
      glsl_base_type_is_integer(type_without_array->base_type);
   const bool is_struct = type_without_array->is_struct();
   if (is_struct) {
      /* structs don't have a defined underlying base type so just treat all
       * component slots as used and set the bit size to 0. If there is
       * location aliasing, we'll fail anyway later.
       */
      last_comp = 4;
      base_type_bit_size = 0;
   } else {
      unsigned dmul = type_without_array->is_64bit() ? 2 : 1;
      last_comp = component + type_without_array->vector_elements * dmul;
      base_type_bit_size =
         glsl_base_type_get_bit_size(type_without_array->base_type);
   }

   while (location < location_limit) {
      unsigned comp = 0;
      while (comp < 4) {
         struct explicit_location_info *info =
            &explicit_locations[location][comp];

         if (info->var) {
            if (info->var->type->without_array()->is_struct() || is_struct) {
               /* Structs cannot share location since they are incompatible
                * with any other underlying numerical type.
                */
               linker_error(prog,
                            "%s shader has multiple %sputs sharing the "
                            "same location that don't have the same "
                            "underlying numerical type. Struct variable '%s', "
                            "location %u\n",
                            _mesa_shader_stage_to_string(stage),
                            var->data.mode == ir_var_shader_in ? "in" : "out",
                            is_struct ? var->name : info->var->name,
                            location);
               return false;
            } else if (comp >= component && comp < last_comp) {
               /* Component aliasing is not allowed */
               linker_error(prog,
                            "%s shader has multiple %sputs explicitly "
                            "assigned to location %d and component %d\n",
                            _mesa_shader_stage_to_string(stage),
                            var->data.mode == ir_var_shader_in ? "in" : "out",
                            location, comp);
               return false;
            } else {
               /* From the OpenGL 4.60.5 spec, section 4.4.1 Input Layout
                * Qualifiers, Page 67, (Location aliasing):
                *
                *   " Further, when location aliasing, the aliases sharing the
                *     location must have the same underlying numerical type
                *     and bit width (floating-point or integer, 32-bit versus
                *     64-bit, etc.) and the same auxiliary storage and
                *     interpolation qualification."
                */

               /* If the underlying numerical type isn't integer, implicitly
                * it will be float or else we would have failed by now.
                */
               if (info->base_type_is_integer != base_type_is_integer) {
                  linker_error(prog,
                               "%s shader has multiple %sputs sharing the "
                               "same location that don't have the same "
                               "underlying numerical type. Location %u "
                               "component %u.\n",
                               _mesa_shader_stage_to_string(stage),
                               var->data.mode == ir_var_shader_in ?
                               "in" : "out", location, comp);
                  return false;
               }

               if (info->base_type_bit_size != base_type_bit_size) {
                  linker_error(prog,
                               "%s shader has multiple %sputs sharing the "
                               "same location that don't have the same "
                               "underlying numerical bit size. Location %u "
                               "component %u.\n",
                               _mesa_shader_stage_to_string(stage),
                               var->data.mode == ir_var_shader_in ?
                               "in" : "out", location, comp);
                  return false;
               }

               if (info->interpolation != interpolation) {
                  linker_error(prog,
                               "%s shader has multiple %sputs sharing the "
                               "same location that don't have the same "
                               "interpolation qualification. Location %u "
                               "component %u.\n",
                               _mesa_shader_stage_to_string(stage),
                               var->data.mode == ir_var_shader_in ?
                               "in" : "out", location, comp);
                  return false;
               }

               if (info->centroid != centroid ||
                   info->sample != sample ||
                   info->patch != patch) {
                  linker_error(prog,
                               "%s shader has multiple %sputs sharing the "
                               "same location that don't have the same "
                               "auxiliary storage qualification. Location %u "
                               "component %u.\n",
                               _mesa_shader_stage_to_string(stage),
                               var->data.mode == ir_var_shader_in ?
                               "in" : "out", location, comp);
                  return false;
               }
            }
         } else if (comp >= component && comp < last_comp) {
            info->var = var;
            info->base_type_is_integer = base_type_is_integer;
            info->base_type_bit_size = base_type_bit_size;
            info->interpolation = interpolation;
            info->centroid = centroid;
            info->sample = sample;
            info->patch = patch;
         }

         comp++;

         /* We need to do some special handling for doubles as dvec3 and
          * dvec4 consume two consecutive locations. We don't need to
          * worry about components beginning at anything other than 0 as
          * the spec does not allow this for dvec3 and dvec4.
          */
         if (comp == 4 && last_comp > 4) {
            last_comp = last_comp - 4;
            /* Bump location index and reset the component index */
            location++;
            comp = 0;
            component = 0;
         }
      }

      location++;
   }

   return true;
}

static bool
validate_explicit_variable_location(struct gl_context *ctx,
                                    struct explicit_location_info explicit_locations[][4],
                                    ir_variable *var,
                                    gl_shader_program *prog,
                                    gl_linked_shader *sh)
{
   const glsl_type *type = get_varying_type(var, sh->Stage);
   unsigned num_elements = type->count_attribute_slots(false);
   unsigned idx = compute_variable_location_slot(var, sh->Stage);
   unsigned slot_limit = idx + num_elements;

   /* Vertex shader inputs and fragment shader outputs are validated in
    * assign_attribute_or_color_locations() so we should not attempt to
    * validate them again here.
    */
   unsigned slot_max;
   if (var->data.mode == ir_var_shader_out) {
      assert(sh->Stage != MESA_SHADER_FRAGMENT);
      slot_max =
         ctx->Const.Program[sh->Stage].MaxOutputComponents / 4;
   } else {
      assert(var->data.mode == ir_var_shader_in);
      assert(sh->Stage != MESA_SHADER_VERTEX);
      slot_max =
         ctx->Const.Program[sh->Stage].MaxInputComponents / 4;
   }

   if (slot_limit > slot_max) {
      linker_error(prog,
                   "Invalid location %u in %s shader\n",
                   idx, _mesa_shader_stage_to_string(sh->Stage));
      return false;
   }

   const glsl_type *type_without_array = type->without_array();
   if (type_without_array->is_interface()) {
      for (unsigned i = 0; i < type_without_array->length; i++) {
         glsl_struct_field *field = &type_without_array->fields.structure[i];
         unsigned field_location = field->location -
            (field->patch ? VARYING_SLOT_PATCH0 : VARYING_SLOT_VAR0);
         if (!check_location_aliasing(explicit_locations, var,
                                      field_location,
                                      0, field_location + 1,
                                      field->type,
                                      field->interpolation,
                                      field->centroid,
                                      field->sample,
                                      field->patch,
                                      prog, sh->Stage)) {
            return false;
         }
      }
   } else if (!check_location_aliasing(explicit_locations, var,
                                       idx, var->data.location_frac,
                                       slot_limit, type,
                                       var->data.interpolation,
                                       var->data.centroid,
                                       var->data.sample,
                                       var->data.patch,
                                       prog, sh->Stage)) {
      return false;
   }

   return true;
}

/**
 * Validate explicit locations for the inputs to the first stage and the
 * outputs of the last stage in a program, if those are not the VS and FS
 * shaders.
 */
void
validate_first_and_last_interface_explicit_locations(struct gl_context *ctx,
                                                     struct gl_shader_program *prog,
                                                     gl_shader_stage first_stage,
                                                     gl_shader_stage last_stage)
{
   /* VS inputs and FS outputs are validated in
    * assign_attribute_or_color_locations()
    */
   bool validate_first_stage = first_stage != MESA_SHADER_VERTEX;
   bool validate_last_stage = last_stage != MESA_SHADER_FRAGMENT;
   if (!validate_first_stage && !validate_last_stage)
      return;

   struct explicit_location_info explicit_locations[MAX_VARYING][4];

   gl_shader_stage stages[2] = { first_stage, last_stage };
   bool validate_stage[2] = { validate_first_stage, validate_last_stage };
   ir_variable_mode var_direction[2] = { ir_var_shader_in, ir_var_shader_out };

   for (unsigned i = 0; i < 2; i++) {
      if (!validate_stage[i])
         continue;

      gl_shader_stage stage = stages[i];

      gl_linked_shader *sh = prog->_LinkedShaders[stage];
      assert(sh);

      memset(explicit_locations, 0, sizeof(explicit_locations));

      foreach_in_list(ir_instruction, node, sh->ir) {
         ir_variable *const var = node->as_variable();

         if (var == NULL ||
             !var->data.explicit_location ||
             var->data.location < VARYING_SLOT_VAR0 ||
             var->data.mode != var_direction[i])
            continue;

         if (!validate_explicit_variable_location(
               ctx, explicit_locations, var, prog, sh)) {
            return;
         }
      }
   }
}

/**
 * Validate that outputs from one stage match inputs of another
 */
void
cross_validate_outputs_to_inputs(struct gl_context *ctx,
                                 struct gl_shader_program *prog,
                                 gl_linked_shader *producer,
                                 gl_linked_shader *consumer)
{
   glsl_symbol_table parameters;
   struct explicit_location_info output_explicit_locations[MAX_VARYING][4] = {};
   struct explicit_location_info input_explicit_locations[MAX_VARYING][4] = {};

   /* Find all shader outputs in the "producer" stage.
    */
   foreach_in_list(ir_instruction, node, producer->ir) {
      ir_variable *const var = node->as_variable();

      if (var == NULL || var->data.mode != ir_var_shader_out)
         continue;

      if (!var->data.explicit_location
          || var->data.location < VARYING_SLOT_VAR0)
         parameters.add_variable(var);
      else {
         /* User-defined varyings with explicit locations are handled
          * differently because they do not need to have matching names.
          */
         if (!validate_explicit_variable_location(ctx,
                                                  output_explicit_locations,
                                                  var, prog, producer)) {
            return;
         }
      }
   }


   /* Find all shader inputs in the "consumer" stage.  Any variables that have
    * matching outputs already in the symbol table must have the same type and
    * qualifiers.
    *
    * Exception: if the consumer is the geometry shader, then the inputs
    * should be arrays and the type of the array element should match the type
    * of the corresponding producer output.
    */
   foreach_in_list(ir_instruction, node, consumer->ir) {
      ir_variable *const input = node->as_variable();

      if (input == NULL || input->data.mode != ir_var_shader_in)
         continue;

      if (strcmp(input->name, "gl_Color") == 0 && input->data.used) {
         const ir_variable *const front_color =
            parameters.get_variable("gl_FrontColor");

         const ir_variable *const back_color =
            parameters.get_variable("gl_BackColor");

         cross_validate_front_and_back_color(ctx, prog, input,
                                             front_color, back_color,
                                             consumer->Stage, producer->Stage);
      } else if (strcmp(input->name, "gl_SecondaryColor") == 0 && input->data.used) {
         const ir_variable *const front_color =
            parameters.get_variable("gl_FrontSecondaryColor");

         const ir_variable *const back_color =
            parameters.get_variable("gl_BackSecondaryColor");

         cross_validate_front_and_back_color(ctx, prog, input,
                                             front_color, back_color,
                                             consumer->Stage, producer->Stage);
      } else {
         /* The rules for connecting inputs and outputs change in the presence
          * of explicit locations.  In this case, we no longer care about the
          * names of the variables.  Instead, we care only about the
          * explicitly assigned location.
          */
         ir_variable *output = NULL;
         if (input->data.explicit_location
             && input->data.location >= VARYING_SLOT_VAR0) {

            const glsl_type *type = get_varying_type(input, consumer->Stage);
            unsigned num_elements = type->count_attribute_slots(false);
            unsigned idx =
               compute_variable_location_slot(input, consumer->Stage);
            unsigned slot_limit = idx + num_elements;

            if (!validate_explicit_variable_location(ctx,
                                                     input_explicit_locations,
                                                     input, prog, consumer)) {
               return;
            }

            while (idx < slot_limit) {
               if (idx >= MAX_VARYING) {
                  linker_error(prog,
                               "Invalid location %u in %s shader\n", idx,
                               _mesa_shader_stage_to_string(consumer->Stage));
                  return;
               }

               output = output_explicit_locations[idx][input->data.location_frac].var;

               if (output == NULL) {
                  /* A linker failure should only happen when there is no
                   * output declaration and there is Static Use of the
                   * declared input.
                   */
                  if (input->data.used) {
                     linker_error(prog,
                                  "%s shader input `%s' with explicit location "
                                  "has no matching output\n",
                                  _mesa_shader_stage_to_string(consumer->Stage),
                                  input->name);
                     break;
                  }
               } else if (input->data.location != output->data.location) {
                  linker_error(prog,
                               "%s shader input `%s' with explicit location "
                               "has no matching output\n",
                               _mesa_shader_stage_to_string(consumer->Stage),
                               input->name);
                  break;
               }
               idx++;
            }
         } else {
            output = parameters.get_variable(input->name);
         }

         if (output != NULL) {
            /* Interface blocks have their own validation elsewhere so don't
             * try validating them here.
             */
            if (!(input->get_interface_type() &&
                  output->get_interface_type()))
               cross_validate_types_and_qualifiers(ctx, prog, input, output,
                                                   consumer->Stage,
                                                   producer->Stage);
         } else {
            /* Check for input vars with unmatched output vars in prev stage
             * taking into account that interface blocks could have a matching
             * output but with different name, so we ignore them.
             */
            assert(!input->data.assigned);
            if (input->data.used && !input->get_interface_type() &&
                !input->data.explicit_location)
               linker_error(prog,
                            "%s shader input `%s' "
                            "has no matching output in the previous stage\n",
                            _mesa_shader_stage_to_string(consumer->Stage),
                            input->name);
         }
      }
   }
}

/**
 * Demote shader inputs and outputs that are not used in other stages, and
 * remove them via dead code elimination.
 */
static void
remove_unused_shader_inputs_and_outputs(bool is_separate_shader_object,
                                        gl_linked_shader *sh,
                                        enum ir_variable_mode mode)
{
   if (is_separate_shader_object)
      return;

   foreach_in_list(ir_instruction, node, sh->ir) {
      ir_variable *const var = node->as_variable();

      if (var == NULL || var->data.mode != int(mode))
         continue;

      /* A shader 'in' or 'out' variable is only really an input or output if
       * its value is used by other shader stages. This will cause the
       * variable to have a location assigned.
       */
      if (var->data.is_unmatched_generic_inout && !var->data.is_xfb_only) {
         assert(var->data.mode != ir_var_temporary);

         /* Assign zeros to demoted inputs to allow more optimizations. */
         if (var->data.mode == ir_var_shader_in && !var->constant_value)
            var->constant_value = ir_constant::zero(var, var->type);

         var->data.mode = ir_var_auto;
      }
   }

   /* Eliminate code that is now dead due to unused inputs/outputs being
    * demoted.
    */
   while (do_dead_code(sh->ir, false))
      ;

}

/**
 * Initialize this object based on a string that was passed to
 * glTransformFeedbackVaryings.
 *
 * If the input is mal-formed, this call still succeeds, but it sets
 * this->var_name to a mal-formed input, so tfeedback_decl::find_output_var()
 * will fail to find any matching variable.
 */
void
tfeedback_decl::init(struct gl_context *ctx, const void *mem_ctx,
                     const char *input)
{
   /* We don't have to be pedantic about what is a valid GLSL variable name,
    * because any variable with an invalid name can't exist in the IR anyway.
    */

   this->location = -1;
   this->orig_name = input;
   this->lowered_builtin_array_variable = none;
   this->skip_components = 0;
   this->next_buffer_separator = false;
   this->matched_candidate = NULL;
   this->stream_id = 0;
   this->buffer = 0;
   this->offset = 0;

   if (ctx->Extensions.ARB_transform_feedback3) {
      /* Parse gl_NextBuffer. */
      if (strcmp(input, "gl_NextBuffer") == 0) {
         this->next_buffer_separator = true;
         return;
      }

      /* Parse gl_SkipComponents. */
      if (strcmp(input, "gl_SkipComponents1") == 0)
         this->skip_components = 1;
      else if (strcmp(input, "gl_SkipComponents2") == 0)
         this->skip_components = 2;
      else if (strcmp(input, "gl_SkipComponents3") == 0)
         this->skip_components = 3;
      else if (strcmp(input, "gl_SkipComponents4") == 0)
         this->skip_components = 4;

      if (this->skip_components)
         return;
   }

   /* Parse a declaration. */
   const char *base_name_end;
   long subscript = parse_program_resource_name(input, &base_name_end);
   this->var_name = ralloc_strndup(mem_ctx, input, base_name_end - input);
   if (this->var_name == NULL) {
      _mesa_error_no_memory(__func__);
      return;
   }

   if (subscript >= 0) {
      this->array_subscript = subscript;
      this->is_subscripted = true;
   } else {
      this->is_subscripted = false;
   }

   /* For drivers that lower gl_ClipDistance to gl_ClipDistanceMESA, this
    * class must behave specially to account for the fact that gl_ClipDistance
    * is converted from a float[8] to a vec4[2].
    */
   if (ctx->Const.ShaderCompilerOptions[MESA_SHADER_VERTEX].LowerCombinedClipCullDistance &&
       strcmp(this->var_name, "gl_ClipDistance") == 0) {
      this->lowered_builtin_array_variable = clip_distance;
   }
   if (ctx->Const.ShaderCompilerOptions[MESA_SHADER_VERTEX].LowerCombinedClipCullDistance &&
       strcmp(this->var_name, "gl_CullDistance") == 0) {
      this->lowered_builtin_array_variable = cull_distance;
   }

   if (ctx->Const.LowerTessLevel &&
       (strcmp(this->var_name, "gl_TessLevelOuter") == 0))
      this->lowered_builtin_array_variable = tess_level_outer;
   if (ctx->Const.LowerTessLevel &&
       (strcmp(this->var_name, "gl_TessLevelInner") == 0))
      this->lowered_builtin_array_variable = tess_level_inner;
}


/**
 * Determine whether two tfeedback_decl objects refer to the same variable and
 * array index (if applicable).
 */
bool
tfeedback_decl::is_same(const tfeedback_decl &x, const tfeedback_decl &y)
{
   assert(x.is_varying() && y.is_varying());

   if (strcmp(x.var_name, y.var_name) != 0)
      return false;
   if (x.is_subscripted != y.is_subscripted)
      return false;
   if (x.is_subscripted && x.array_subscript != y.array_subscript)
      return false;
   return true;
}


/**
 * Assign a location and stream ID for this tfeedback_decl object based on the
 * transform feedback candidate found by find_candidate.
 *
 * If an error occurs, the error is reported through linker_error() and false
 * is returned.
 */
bool
tfeedback_decl::assign_location(struct gl_context *ctx,
                                struct gl_shader_program *prog)
{
   assert(this->is_varying());

   unsigned fine_location
      = this->matched_candidate->toplevel_var->data.location * 4
      + this->matched_candidate->toplevel_var->data.location_frac
      + this->matched_candidate->offset;
   const unsigned dmul =
      this->matched_candidate->type->without_array()->is_64bit() ? 2 : 1;

   if (this->matched_candidate->type->is_array()) {
      /* Array variable */
      const unsigned matrix_cols =
         this->matched_candidate->type->fields.array->matrix_columns;
      const unsigned vector_elements =
         this->matched_candidate->type->fields.array->vector_elements;
      unsigned actual_array_size;
      switch (this->lowered_builtin_array_variable) {
      case clip_distance:
         actual_array_size = prog->last_vert_prog ?
            prog->last_vert_prog->info.clip_distance_array_size : 0;
         break;
      case cull_distance:
         actual_array_size = prog->last_vert_prog ?
            prog->last_vert_prog->info.cull_distance_array_size : 0;
         break;
      case tess_level_outer:
         actual_array_size = 4;
         break;
      case tess_level_inner:
         actual_array_size = 2;
         break;
      case none:
      default:
         actual_array_size = this->matched_candidate->type->array_size();
         break;
      }

      if (this->is_subscripted) {
         /* Check array bounds. */
         if (this->array_subscript >= actual_array_size) {
            linker_error(prog, "Transform feedback varying %s has index "
                         "%i, but the array size is %u.",
                         this->orig_name, this->array_subscript,
                         actual_array_size);
            return false;
         }
         unsigned array_elem_size = this->lowered_builtin_array_variable ?
            1 : vector_elements * matrix_cols * dmul;
         fine_location += array_elem_size * this->array_subscript;
         this->size = 1;
      } else {
         this->size = actual_array_size;
      }
      this->vector_elements = vector_elements;
      this->matrix_columns = matrix_cols;
      if (this->lowered_builtin_array_variable)
         this->type = GL_FLOAT;
      else
         this->type = this->matched_candidate->type->fields.array->gl_type;
   } else {
      /* Regular variable (scalar, vector, or matrix) */
      if (this->is_subscripted) {
         linker_error(prog, "Transform feedback varying %s requested, "
                      "but %s is not an array.",
                      this->orig_name, this->var_name);
         return false;
      }
      this->size = 1;
      this->vector_elements = this->matched_candidate->type->vector_elements;
      this->matrix_columns = this->matched_candidate->type->matrix_columns;
      this->type = this->matched_candidate->type->gl_type;
   }
   this->location = fine_location / 4;
   this->location_frac = fine_location % 4;

   /* From GL_EXT_transform_feedback:
    *   A program will fail to link if:
    *
    *   * the total number of components to capture in any varying
    *     variable in <varyings> is greater than the constant
    *     MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS_EXT and the
    *     buffer mode is SEPARATE_ATTRIBS_EXT;
    */
   if (prog->TransformFeedback.BufferMode == GL_SEPARATE_ATTRIBS &&
       this->num_components() >
       ctx->Const.MaxTransformFeedbackSeparateComponents) {
      linker_error(prog, "Transform feedback varying %s exceeds "
                   "MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS.",
                   this->orig_name);
      return false;
   }

   /* Only transform feedback varyings can be assigned to non-zero streams,
    * so assign the stream id here.
    */
   this->stream_id = this->matched_candidate->toplevel_var->data.stream;

   unsigned array_offset = this->array_subscript * 4 * dmul;
   unsigned struct_offset = this->matched_candidate->offset * 4 * dmul;
   this->buffer = this->matched_candidate->toplevel_var->data.xfb_buffer;
   this->offset = this->matched_candidate->toplevel_var->data.offset +
      array_offset + struct_offset;

   return true;
}


unsigned
tfeedback_decl::get_num_outputs() const
{
   if (!this->is_varying()) {
      return 0;
   }
   return (this->num_components() + this->location_frac + 3)/4;
}


/**
 * Update gl_transform_feedback_info to reflect this tfeedback_decl.
 *
 * If an error occurs, the error is reported through linker_error() and false
 * is returned.
 */
bool
tfeedback_decl::store(struct gl_context *ctx, struct gl_shader_program *prog,
                      struct gl_transform_feedback_info *info,
                      unsigned buffer, unsigned buffer_index,
                      const unsigned max_outputs,
                      BITSET_WORD *used_components[MAX_FEEDBACK_BUFFERS],
                      bool *explicit_stride, bool has_xfb_qualifiers,
                      const void* mem_ctx) const
{
   unsigned xfb_offset = 0;
   unsigned size = this->size;
   /* Handle gl_SkipComponents. */
   if (this->skip_components) {
      info->Buffers[buffer].Stride += this->skip_components;
      size = this->skip_components;
      goto store_varying;
   }

   if (this->next_buffer_separator) {
      size = 0;
      goto store_varying;
   }

   if (has_xfb_qualifiers) {
      xfb_offset = this->offset / 4;
   } else {
      xfb_offset = info->Buffers[buffer].Stride;
   }
   info->Varyings[info->NumVarying].Offset = xfb_offset * 4;

   {
      unsigned location = this->location;
      unsigned location_frac = this->location_frac;
      unsigned num_components = this->num_components();

      /* From GL_EXT_transform_feedback:
       *
       *   " A program will fail to link if:
       *
       *       * the total number of components to capture is greater than the
       *         constant MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS_EXT
       *         and the buffer mode is INTERLEAVED_ATTRIBS_EXT."
       *
       * From GL_ARB_enhanced_layouts:
       *
       *   " The resulting stride (implicit or explicit) must be less than or
       *     equal to the implementation-dependent constant
       *     gl_MaxTransformFeedbackInterleavedComponents."
       */
      if ((prog->TransformFeedback.BufferMode == GL_INTERLEAVED_ATTRIBS ||
           has_xfb_qualifiers) &&
          xfb_offset + num_components >
          ctx->Const.MaxTransformFeedbackInterleavedComponents) {
         linker_error(prog,
                      "The MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS "
                      "limit has been exceeded.");
         return false;
      }

      /* From the OpenGL 4.60.5 spec, section 4.4.2. Output Layout Qualifiers,
       * Page 76, (Transform Feedback Layout Qualifiers):
       *
       *   " No aliasing in output buffers is allowed: It is a compile-time or
       *     link-time error to specify variables with overlapping transform
       *     feedback offsets."
       */
      const unsigned max_components =
         ctx->Const.MaxTransformFeedbackInterleavedComponents;
      const unsigned first_component = xfb_offset;
      const unsigned last_component = xfb_offset + num_components - 1;
      const unsigned start_word = BITSET_BITWORD(first_component);
      const unsigned end_word = BITSET_BITWORD(last_component);
      BITSET_WORD *used;
      assert(last_component < max_components);

      if (!used_components[buffer]) {
         used_components[buffer] =
            rzalloc_array(mem_ctx, BITSET_WORD, BITSET_WORDS(max_components));
      }
      used = used_components[buffer];

      for (unsigned word = start_word; word <= end_word; word++) {
         unsigned start_range = 0;
         unsigned end_range = BITSET_WORDBITS - 1;

         if (word == start_word)
            start_range = first_component % BITSET_WORDBITS;

         if (word == end_word)
            end_range = last_component % BITSET_WORDBITS;

         if (used[word] & BITSET_RANGE(start_range, end_range)) {
            linker_error(prog,
                         "variable '%s', xfb_offset (%d) is causing aliasing.",
                         this->orig_name, xfb_offset * 4);
            return false;
         }
         used[word] |= BITSET_RANGE(start_range, end_range);
      }

      while (num_components > 0) {
         unsigned output_size = MIN2(num_components, 4 - location_frac);
         assert((info->NumOutputs == 0 && max_outputs == 0) ||
                info->NumOutputs < max_outputs);

         /* From the ARB_enhanced_layouts spec:
          *
          *    "If such a block member or variable is not written during a shader
          *    invocation, the buffer contents at the assigned offset will be
          *    undefined.  Even if there are no static writes to a variable or
          *    member that is assigned a transform feedback offset, the space is
          *    still allocated in the buffer and still affects the stride."
          */
         if (this->is_varying_written()) {
            info->Outputs[info->NumOutputs].ComponentOffset = location_frac;
            info->Outputs[info->NumOutputs].OutputRegister = location;
            info->Outputs[info->NumOutputs].NumComponents = output_size;
            info->Outputs[info->NumOutputs].StreamId = stream_id;
            info->Outputs[info->NumOutputs].OutputBuffer = buffer;
            info->Outputs[info->NumOutputs].DstOffset = xfb_offset;
            ++info->NumOutputs;
         }
         info->Buffers[buffer].Stream = this->stream_id;
         xfb_offset += output_size;

         num_components -= output_size;
         location++;
         location_frac = 0;
      }
   }

   if (explicit_stride && explicit_stride[buffer]) {
      if (this->is_64bit() && info->Buffers[buffer].Stride % 2) {
         linker_error(prog, "invalid qualifier xfb_stride=%d must be a "
                      "multiple of 8 as its applied to a type that is or "
                      "contains a double.",
                      info->Buffers[buffer].Stride * 4);
         return false;
      }

      if (xfb_offset > info->Buffers[buffer].Stride) {
         linker_error(prog, "xfb_offset (%d) overflows xfb_stride (%d) for "
                      "buffer (%d)", xfb_offset * 4,
                      info->Buffers[buffer].Stride * 4, buffer);
         return false;
      }
   } else {
      info->Buffers[buffer].Stride = xfb_offset;
   }

 store_varying:
   info->Varyings[info->NumVarying].Name = ralloc_strdup(prog,
                                                         this->orig_name);
   info->Varyings[info->NumVarying].Type = this->type;
   info->Varyings[info->NumVarying].Size = size;
   info->Varyings[info->NumVarying].BufferIndex = buffer_index;
   info->NumVarying++;
   info->Buffers[buffer].NumVaryings++;

   return true;
}


const tfeedback_candidate *
tfeedback_decl::find_candidate(gl_shader_program *prog,
                               hash_table *tfeedback_candidates)
{
   const char *name = this->var_name;
   switch (this->lowered_builtin_array_variable) {
   case none:
      name = this->var_name;
      break;
   case clip_distance:
      name = "gl_ClipDistanceMESA";
      break;
   case cull_distance:
      name = "gl_CullDistanceMESA";
      break;
   case tess_level_outer:
      name = "gl_TessLevelOuterMESA";
      break;
   case tess_level_inner:
      name = "gl_TessLevelInnerMESA";
      break;
   }
   hash_entry *entry = _mesa_hash_table_search(tfeedback_candidates, name);

   this->matched_candidate = entry ?
         (const tfeedback_candidate *) entry->data : NULL;

   if (!this->matched_candidate) {
      /* From GL_EXT_transform_feedback:
       *   A program will fail to link if:
       *
       *   * any variable name specified in the <varyings> array is not
       *     declared as an output in the geometry shader (if present) or
       *     the vertex shader (if no geometry shader is present);
       */
      linker_error(prog, "Transform feedback varying %s undeclared.",
                   this->orig_name);
   }

   return this->matched_candidate;
}

/**
 * Force a candidate over the previously matched one. It happens when a new
 * varying needs to be created to match the xfb declaration, for example,
 * to fullfil an alignment criteria.
 */
void
tfeedback_decl::set_lowered_candidate(const tfeedback_candidate *candidate)
{
   this->matched_candidate = candidate;

   /* The subscript part is no longer relevant */
   this->is_subscripted = false;
   this->array_subscript = 0;
}


/**
 * Parse all the transform feedback declarations that were passed to
 * glTransformFeedbackVaryings() and store them in tfeedback_decl objects.
 *
 * If an error occurs, the error is reported through linker_error() and false
 * is returned.
 */
static bool
parse_tfeedback_decls(struct gl_context *ctx, struct gl_shader_program *prog,
                      const void *mem_ctx, unsigned num_names,
                      char **varying_names, tfeedback_decl *decls)
{
   for (unsigned i = 0; i < num_names; ++i) {
      decls[i].init(ctx, mem_ctx, varying_names[i]);

      if (!decls[i].is_varying())
         continue;

      /* From GL_EXT_transform_feedback:
       *   A program will fail to link if:
       *
       *   * any two entries in the <varyings> array specify the same varying
       *     variable;
       *
       * We interpret this to mean "any two entries in the <varyings> array
       * specify the same varying variable and array index", since transform
       * feedback of arrays would be useless otherwise.
       */
      for (unsigned j = 0; j < i; ++j) {
         if (decls[j].is_varying()) {
            if (tfeedback_decl::is_same(decls[i], decls[j])) {
               linker_error(prog, "Transform feedback varying %s specified "
                            "more than once.", varying_names[i]);
               return false;
            }
         }
      }
   }
   return true;
}


static int
cmp_xfb_offset(const void * x_generic, const void * y_generic)
{
   tfeedback_decl *x = (tfeedback_decl *) x_generic;
   tfeedback_decl *y = (tfeedback_decl *) y_generic;

   if (x->get_buffer() != y->get_buffer())
      return x->get_buffer() - y->get_buffer();
   return x->get_offset() - y->get_offset();
}

/**
 * Store transform feedback location assignments into
 * prog->sh.LinkedTransformFeedback based on the data stored in
 * tfeedback_decls.
 *
 * If an error occurs, the error is reported through linker_error() and false
 * is returned.
 */
static bool
store_tfeedback_info(struct gl_context *ctx, struct gl_shader_program *prog,
                     unsigned num_tfeedback_decls,
                     tfeedback_decl *tfeedback_decls, bool has_xfb_qualifiers,
                     const void *mem_ctx)
{
   if (!prog->last_vert_prog)
      return true;

   /* Make sure MaxTransformFeedbackBuffers is less than 32 so the bitmask for
    * tracking the number of buffers doesn't overflow.
    */
   assert(ctx->Const.MaxTransformFeedbackBuffers < 32);

   bool separate_attribs_mode =
      prog->TransformFeedback.BufferMode == GL_SEPARATE_ATTRIBS;

   struct gl_program *xfb_prog = prog->last_vert_prog;
   xfb_prog->sh.LinkedTransformFeedback =
      rzalloc(xfb_prog, struct gl_transform_feedback_info);

   /* The xfb_offset qualifier does not have to be used in increasing order
    * however some drivers expect to receive the list of transform feedback
    * declarations in order so sort it now for convenience.
    */
   if (has_xfb_qualifiers) {
      qsort(tfeedback_decls, num_tfeedback_decls, sizeof(*tfeedback_decls),
            cmp_xfb_offset);
   }

   xfb_prog->sh.LinkedTransformFeedback->Varyings =
      rzalloc_array(xfb_prog, struct gl_transform_feedback_varying_info,
                    num_tfeedback_decls);

   unsigned num_outputs = 0;
   for (unsigned i = 0; i < num_tfeedback_decls; ++i) {
      if (tfeedback_decls[i].is_varying_written())
         num_outputs += tfeedback_decls[i].get_num_outputs();
   }

   xfb_prog->sh.LinkedTransformFeedback->Outputs =
      rzalloc_array(xfb_prog, struct gl_transform_feedback_output,
                    num_outputs);

   unsigned num_buffers = 0;
   unsigned buffers = 0;
   BITSET_WORD *used_components[MAX_FEEDBACK_BUFFERS] = {};

   if (!has_xfb_qualifiers && separate_attribs_mode) {
      /* GL_SEPARATE_ATTRIBS */
      for (unsigned i = 0; i < num_tfeedback_decls; ++i) {
         if (!tfeedback_decls[i].store(ctx, prog,
                                       xfb_prog->sh.LinkedTransformFeedback,
                                       num_buffers, num_buffers, num_outputs,
                                       used_components, NULL,
                                       has_xfb_qualifiers, mem_ctx))
            return false;

         buffers |= 1 << num_buffers;
         num_buffers++;
      }
   }
   else {
      /* GL_INVERLEAVED_ATTRIBS */
      int buffer_stream_id = -1;
      unsigned buffer =
         num_tfeedback_decls ? tfeedback_decls[0].get_buffer() : 0;
      bool explicit_stride[MAX_FEEDBACK_BUFFERS] = { false };

      /* Apply any xfb_stride global qualifiers */
      if (has_xfb_qualifiers) {
         for (unsigned j = 0; j < MAX_FEEDBACK_BUFFERS; j++) {
            if (prog->TransformFeedback.BufferStride[j]) {
               explicit_stride[j] = true;
               xfb_prog->sh.LinkedTransformFeedback->Buffers[j].Stride =
                  prog->TransformFeedback.BufferStride[j] / 4;
            }
         }
      }

      for (unsigned i = 0; i < num_tfeedback_decls; ++i) {
         if (has_xfb_qualifiers &&
             buffer != tfeedback_decls[i].get_buffer()) {
            /* we have moved to the next buffer so reset stream id */
            buffer_stream_id = -1;
            num_buffers++;
         }

         if (tfeedback_decls[i].is_next_buffer_separator()) {
            if (!tfeedback_decls[i].store(ctx, prog,
                                          xfb_prog->sh.LinkedTransformFeedback,
                                          buffer, num_buffers, num_outputs,
                                          used_components, explicit_stride,
                                          has_xfb_qualifiers, mem_ctx))
               return false;
            num_buffers++;
            buffer_stream_id = -1;
            continue;
         }

         if (has_xfb_qualifiers) {
            buffer = tfeedback_decls[i].get_buffer();
         } else {
            buffer = num_buffers;
         }

         if (tfeedback_decls[i].is_varying()) {
            if (buffer_stream_id == -1)  {
               /* First varying writing to this buffer: remember its stream */
               buffer_stream_id = (int) tfeedback_decls[i].get_stream_id();

               /* Only mark a buffer as active when there is a varying
                * attached to it. This behaviour is based on a revised version
                * of section 13.2.2 of the GL 4.6 spec.
                */
               buffers |= 1 << buffer;
            } else if (buffer_stream_id !=
                       (int) tfeedback_decls[i].get_stream_id()) {
               /* Varying writes to the same buffer from a different stream */
               linker_error(prog,
                            "Transform feedback can't capture varyings belonging "
                            "to different vertex streams in a single buffer. "
                            "Varying %s writes to buffer from stream %u, other "
                            "varyings in the same buffer write from stream %u.",
                            tfeedback_decls[i].name(),
                            tfeedback_decls[i].get_stream_id(),
                            buffer_stream_id);
               return false;
            }
         }

         if (!tfeedback_decls[i].store(ctx, prog,
                                       xfb_prog->sh.LinkedTransformFeedback,
                                       buffer, num_buffers, num_outputs,
                                       used_components, explicit_stride,
                                       has_xfb_qualifiers, mem_ctx))
            return false;
      }
   }

   assert(xfb_prog->sh.LinkedTransformFeedback->NumOutputs == num_outputs);

   xfb_prog->sh.LinkedTransformFeedback->ActiveBuffers = buffers;
   return true;
}

namespace {

/**
 * Data structure recording the relationship between outputs of one shader
 * stage (the "producer") and inputs of another (the "consumer").
 */
class varying_matches
{
public:
   varying_matches(bool disable_varying_packing,
                   bool disable_xfb_packing,
                   bool xfb_enabled,
                   bool enhanced_layouts_enabled,
                   gl_shader_stage producer_stage,
                   gl_shader_stage consumer_stage);
   ~varying_matches();
   void record(ir_variable *producer_var, ir_variable *consumer_var);
   unsigned assign_locations(struct gl_shader_program *prog,
                             uint8_t components[],
                             uint64_t reserved_slots);
   void store_locations() const;

private:
   bool is_varying_packing_safe(const glsl_type *type,
                                const ir_variable *var) const;

   /**
    * If true, this driver disables varying packing, so all varyings need to
    * be aligned on slot boundaries, and take up a number of slots equal to
    * their number of matrix columns times their array size.
    *
    * Packing may also be disabled because our current packing method is not
    * safe in SSO or versions of OpenGL where interpolation qualifiers are not
    * guaranteed to match across stages.
    */
   const bool disable_varying_packing;

   /**
    * If true, this driver disables packing for varyings used by transform
    * feedback.
    */
   const bool disable_xfb_packing;

   /**
    * If true, this driver has transform feedback enabled. The transform
    * feedback code usually requires at least some packing be done even
    * when varying packing is disabled, fortunately where transform feedback
    * requires packing it's safe to override the disabled setting. See
    * is_varying_packing_safe().
    */
   const bool xfb_enabled;

   const bool enhanced_layouts_enabled;

   /**
    * Enum representing the order in which varyings are packed within a
    * packing class.
    *
    * Currently we pack vec4's first, then vec2's, then scalar values, then
    * vec3's.  This order ensures that the only vectors that are at risk of
    * having to be "double parked" (split between two adjacent varying slots)
    * are the vec3's.
    */
   enum packing_order_enum {
      PACKING_ORDER_VEC4,
      PACKING_ORDER_VEC2,
      PACKING_ORDER_SCALAR,
      PACKING_ORDER_VEC3,
   };

   static unsigned compute_packing_class(const ir_variable *var);
   static packing_order_enum compute_packing_order(const ir_variable *var);
   static int match_comparator(const void *x_generic, const void *y_generic);
   static int xfb_comparator(const void *x_generic, const void *y_generic);
   static int not_xfb_comparator(const void *x_generic, const void *y_generic);

   /**
    * Structure recording the relationship between a single producer output
    * and a single consumer input.
    */
   struct match {
      /**
       * Packing class for this varying, computed by compute_packing_class().
       */
      unsigned packing_class;

      /**
       * Packing order for this varying, computed by compute_packing_order().
       */
      packing_order_enum packing_order;
      unsigned num_components;

      /**
       * The output variable in the producer stage.
       */
      ir_variable *producer_var;

      /**
       * The input variable in the consumer stage.
       */
      ir_variable *consumer_var;

      /**
       * The location which has been assigned for this varying.  This is
       * expressed in multiples of a float, with the first generic varying
       * (i.e. the one referred to by VARYING_SLOT_VAR0) represented by the
       * value 0.
       */
      unsigned generic_location;
   } *matches;

   /**
    * The number of elements in the \c matches array that are currently in
    * use.
    */
   unsigned num_matches;

   /**
    * The number of elements that were set aside for the \c matches array when
    * it was allocated.
    */
   unsigned matches_capacity;

   gl_shader_stage producer_stage;
   gl_shader_stage consumer_stage;
};

} /* anonymous namespace */

varying_matches::varying_matches(bool disable_varying_packing,
                                 bool disable_xfb_packing,
                                 bool xfb_enabled,
                                 bool enhanced_layouts_enabled,
                                 gl_shader_stage producer_stage,
                                 gl_shader_stage consumer_stage)
   : disable_varying_packing(disable_varying_packing),
     disable_xfb_packing(disable_xfb_packing),
     xfb_enabled(xfb_enabled),
     enhanced_layouts_enabled(enhanced_layouts_enabled),
     producer_stage(producer_stage),
     consumer_stage(consumer_stage)
{
   /* Note: this initial capacity is rather arbitrarily chosen to be large
    * enough for many cases without wasting an unreasonable amount of space.
    * varying_matches::record() will resize the array if there are more than
    * this number of varyings.
    */
   this->matches_capacity = 8;
   this->matches = (match *)
      malloc(sizeof(*this->matches) * this->matches_capacity);
   this->num_matches = 0;
}


varying_matches::~varying_matches()
{
   free(this->matches);
}


/**
 * Packing is always safe on individual arrays, structures, and matrices. It
 * is also safe if the varying is only used for transform feedback.
 */
bool
varying_matches::is_varying_packing_safe(const glsl_type *type,
                                         const ir_variable *var) const
{
   if (consumer_stage == MESA_SHADER_TESS_EVAL ||
       consumer_stage == MESA_SHADER_TESS_CTRL ||
       producer_stage == MESA_SHADER_TESS_CTRL)
      return false;

   return xfb_enabled && (type->is_array() || type->is_struct() ||
                          type->is_matrix() || var->data.is_xfb_only);
}


/**
 * Record the given producer/consumer variable pair in the list of variables
 * that should later be assigned locations.
 *
 * It is permissible for \c consumer_var to be NULL (this happens if a
 * variable is output by the producer and consumed by transform feedback, but
 * not consumed by the consumer).
 *
 * If \c producer_var has already been paired up with a consumer_var, or
 * producer_var is part of fixed pipeline functionality (and hence already has
 * a location assigned), this function has no effect.
 *
 * Note: as a side effect this function may change the interpolation type of
 * \c producer_var, but only when the change couldn't possibly affect
 * rendering.
 */
void
varying_matches::record(ir_variable *producer_var, ir_variable *consumer_var)
{
   assert(producer_var != NULL || consumer_var != NULL);

   if ((producer_var && (!producer_var->data.is_unmatched_generic_inout ||
       producer_var->data.explicit_location)) ||
       (consumer_var && (!consumer_var->data.is_unmatched_generic_inout ||
       consumer_var->data.explicit_location))) {
      /* Either a location already exists for this variable (since it is part
       * of fixed functionality), or it has already been recorded as part of a
       * previous match.
       */
      return;
   }

   bool needs_flat_qualifier = consumer_var == NULL &&
      (producer_var->type->contains_integer() ||
       producer_var->type->contains_double());

   if (!disable_varying_packing &&
       (!disable_xfb_packing || producer_var  == NULL || !producer_var->data.is_xfb) &&
       (needs_flat_qualifier ||
        (consumer_stage != MESA_SHADER_NONE && consumer_stage != MESA_SHADER_FRAGMENT))) {
      /* Since this varying is not being consumed by the fragment shader, its
       * interpolation type varying cannot possibly affect rendering.
       * Also, this variable is non-flat and is (or contains) an integer
       * or a double.
       * If the consumer stage is unknown, don't modify the interpolation
       * type as it could affect rendering later with separate shaders.
       *
       * lower_packed_varyings requires all integer varyings to flat,
       * regardless of where they appear.  We can trivially satisfy that
       * requirement by changing the interpolation type to flat here.
       */
      if (producer_var) {
         producer_var->data.centroid = false;
         producer_var->data.sample = false;
         producer_var->data.interpolation = INTERP_MODE_FLAT;
      }

      if (consumer_var) {
         consumer_var->data.centroid = false;
         consumer_var->data.sample = false;
         consumer_var->data.interpolation = INTERP_MODE_FLAT;
      }
   }

   if (this->num_matches == this->matches_capacity) {
      this->matches_capacity *= 2;
      this->matches = (match *)
         realloc(this->matches,
                 sizeof(*this->matches) * this->matches_capacity);
   }

   /* We must use the consumer to compute the packing class because in GL4.4+
    * there is no guarantee interpolation qualifiers will match across stages.
    *
    * From Section 4.5 (Interpolation Qualifiers) of the GLSL 4.30 spec:
    *
    *    "The type and presence of interpolation qualifiers of variables with
    *    the same name declared in all linked shaders for the same cross-stage
    *    interface must match, otherwise the link command will fail.
    *
    *    When comparing an output from one stage to an input of a subsequent
    *    stage, the input and output don't match if their interpolation
    *    qualifiers (or lack thereof) are not the same."
    *
    * This text was also in at least revison 7 of the 4.40 spec but is no
    * longer in revision 9 and not in the 4.50 spec.
    */
   const ir_variable *const var = (consumer_var != NULL)
      ? consumer_var : producer_var;
   const gl_shader_stage stage = (consumer_var != NULL)
      ? consumer_stage : producer_stage;
   const glsl_type *type = get_varying_type(var, stage);

   if (producer_var && consumer_var &&
       consumer_var->data.must_be_shader_input) {
      producer_var->data.must_be_shader_input = 1;
   }

   this->matches[this->num_matches].packing_class
      = this->compute_packing_class(var);
   this->matches[this->num_matches].packing_order
      = this->compute_packing_order(var);
   if ((this->disable_varying_packing && !is_varying_packing_safe(type, var)) ||
       (this->disable_xfb_packing && var->data.is_xfb) ||
       var->data.must_be_shader_input) {
      unsigned slots = type->count_attribute_slots(false);
      this->matches[this->num_matches].num_components = slots * 4;
   } else {
      this->matches[this->num_matches].num_components
         = type->component_slots();
   }

   this->matches[this->num_matches].producer_var = producer_var;
   this->matches[this->num_matches].consumer_var = consumer_var;
   this->num_matches++;
   if (producer_var)
      producer_var->data.is_unmatched_generic_inout = 0;
   if (consumer_var)
      consumer_var->data.is_unmatched_generic_inout = 0;
}


/**
 * Choose locations for all of the variable matches that were previously
 * passed to varying_matches::record().
 * \param components  returns array[slot] of number of components used
 *                    per slot (1, 2, 3 or 4)
 * \param reserved_slots  bitmask indicating which varying slots are already
 *                        allocated
 * \return number of slots (4-element vectors) allocated
 */
unsigned
varying_matches::assign_locations(struct gl_shader_program *prog,
                                  uint8_t components[],
                                  uint64_t reserved_slots)
{
   /* If packing has been disabled then we cannot safely sort the varyings by
    * class as it may mean we are using a version of OpenGL where
    * interpolation qualifiers are not guaranteed to be matching across
    * shaders, sorting in this case could result in mismatching shader
    * interfaces.
    * When packing is disabled the sort orders varyings used by transform
    * feedback first, but also depends on *undefined behaviour* of qsort to
    * reverse the order of the varyings. See: xfb_comparator().
    *
    * If packing is only disabled for xfb varyings (mutually exclusive with
    * disable_varying_packing), we then group varyings depending on if they
    * are captured for transform feedback. The same *undefined behaviour* is
    * taken advantage of.
    */
   if (this->disable_varying_packing) {
      /* Only sort varyings that are only used by transform feedback. */
      qsort(this->matches, this->num_matches, sizeof(*this->matches),
            &varying_matches::xfb_comparator);
   } else if (this->disable_xfb_packing) {
      /* Only sort varyings that are NOT used by transform feedback. */
      qsort(this->matches, this->num_matches, sizeof(*this->matches),
            &varying_matches::not_xfb_comparator);
   } else {
      /* Sort varying matches into an order that makes them easy to pack. */
      qsort(this->matches, this->num_matches, sizeof(*this->matches),
            &varying_matches::match_comparator);
   }

   unsigned generic_location = 0;
   unsigned generic_patch_location = MAX_VARYING*4;
   bool previous_var_xfb = false;
   bool previous_var_xfb_only = false;
   unsigned previous_packing_class = ~0u;

   /* For tranform feedback separate mode, we know the number of attributes
    * is <= the number of buffers.  So packing isn't critical.  In fact,
    * packing vec3 attributes can cause trouble because splitting a vec3
    * effectively creates an additional transform feedback output.  The
    * extra TFB output may exceed device driver limits.
    */
   const bool dont_pack_vec3 =
      (prog->TransformFeedback.BufferMode == GL_SEPARATE_ATTRIBS &&
       prog->TransformFeedback.NumVarying > 0);

   for (unsigned i = 0; i < this->num_matches; i++) {
      unsigned *location = &generic_location;
      const ir_variable *var;
      const glsl_type *type;
      bool is_vertex_input = false;

      if (matches[i].consumer_var) {
         var = matches[i].consumer_var;
         type = get_varying_type(var, consumer_stage);
         if (consumer_stage == MESA_SHADER_VERTEX)
            is_vertex_input = true;
      } else {
         var = matches[i].producer_var;
         type = get_varying_type(var, producer_stage);
      }

      if (var->data.patch)
         location = &generic_patch_location;

      /* Advance to the next slot if this varying has a different packing
       * class than the previous one, and we're not already on a slot
       * boundary.
       *
       * Also advance if varying packing is disabled for transform feedback,
       * and previous or current varying is used for transform feedback.
       *
       * Also advance to the next slot if packing is disabled. This makes sure
       * we don't assign varyings the same locations which is possible
       * because we still pack individual arrays, records and matrices even
       * when packing is disabled. Note we don't advance to the next slot if
       * we can pack varyings together that are only used for transform
       * feedback.
       */
      if (var->data.must_be_shader_input ||
          (this->disable_xfb_packing &&
           (previous_var_xfb || var->data.is_xfb)) ||
          (this->disable_varying_packing &&
           !(previous_var_xfb_only && var->data.is_xfb_only)) ||
          (previous_packing_class != this->matches[i].packing_class) ||
          (this->matches[i].packing_order == PACKING_ORDER_VEC3 &&
           dont_pack_vec3)) {
         *location = ALIGN(*location, 4);
      }

      previous_var_xfb = var->data.is_xfb;
      previous_var_xfb_only = var->data.is_xfb_only;
      previous_packing_class = this->matches[i].packing_class;

      /* The number of components taken up by this variable. For vertex shader
       * inputs, we use the number of slots * 4, as they have different
       * counting rules.
       */
      unsigned num_components = is_vertex_input ?
         type->count_attribute_slots(is_vertex_input) * 4 :
         this->matches[i].num_components;

      /* The last slot for this variable, inclusive. */
      unsigned slot_end = *location + num_components - 1;

      /* FIXME: We could be smarter in the below code and loop back over
       * trying to fill any locations that we skipped because we couldn't pack
       * the varying between an explicit location. For now just let the user
       * hit the linking error if we run out of room and suggest they use
       * explicit locations.
       */
      while (slot_end < MAX_VARYING * 4u) {
         const unsigned slots = (slot_end / 4u) - (*location / 4u) + 1;
         const uint64_t slot_mask = ((1ull << slots) - 1) << (*location / 4u);

         assert(slots > 0);

         if ((reserved_slots & slot_mask) == 0) {
            break;
         }

         *location = ALIGN(*location + 1, 4);
         slot_end = *location + num_components - 1;
      }

      if (!var->data.patch && slot_end >= MAX_VARYING * 4u) {
         linker_error(prog, "insufficient contiguous locations available for "
                      "%s it is possible an array or struct could not be "
                      "packed between varyings with explicit locations. Try "
                      "using an explicit location for arrays and structs.",
                      var->name);
      }

      if (slot_end < MAX_VARYINGS_INCL_PATCH * 4u) {
         for (unsigned j = *location / 4u; j < slot_end / 4u; j++)
            components[j] = 4;
         components[slot_end / 4u] = (slot_end & 3) + 1;
      }

      this->matches[i].generic_location = *location;

      *location = slot_end + 1;
   }

   return (generic_location + 3) / 4;
}


/**
 * Update the producer and consumer shaders to reflect the locations
 * assignments that were made by varying_matches::assign_locations().
 */
void
varying_matches::store_locations() const
{
   /* Check is location needs to be packed with lower_packed_varyings() or if
    * we can just use ARB_enhanced_layouts packing.
    */
   bool pack_loc[MAX_VARYINGS_INCL_PATCH] = { 0 };
   const glsl_type *loc_type[MAX_VARYINGS_INCL_PATCH][4] = { {NULL, NULL} };

   for (unsigned i = 0; i < this->num_matches; i++) {
      ir_variable *producer_var = this->matches[i].producer_var;
      ir_variable *consumer_var = this->matches[i].consumer_var;
      unsigned generic_location = this->matches[i].generic_location;
      unsigned slot = generic_location / 4;
      unsigned offset = generic_location % 4;

      if (producer_var) {
         producer_var->data.location = VARYING_SLOT_VAR0 + slot;
         producer_var->data.location_frac = offset;
      }

      if (consumer_var) {
         assert(consumer_var->data.location == -1);
         consumer_var->data.location = VARYING_SLOT_VAR0 + slot;
         consumer_var->data.location_frac = offset;
      }

      /* Find locations suitable for native packing via
       * ARB_enhanced_layouts.
       */
      if (producer_var && consumer_var) {
         if (enhanced_layouts_enabled) {
            const glsl_type *type =
               get_varying_type(producer_var, producer_stage);
            if (type->is_array() || type->is_matrix() || type->is_struct() ||
                type->is_64bit()) {
               unsigned comp_slots = type->component_slots() + offset;
               unsigned slots = comp_slots / 4;
               if (comp_slots % 4)
                  slots += 1;

               for (unsigned j = 0; j < slots; j++) {
                  pack_loc[slot + j] = true;
               }
            } else if (offset + type->vector_elements > 4) {
               pack_loc[slot] = true;
               pack_loc[slot + 1] = true;
            } else {
               loc_type[slot][offset] = type;
            }
         }
      }
   }

   /* Attempt to use ARB_enhanced_layouts for more efficient packing if
    * suitable.
    */
   if (enhanced_layouts_enabled) {
      for (unsigned i = 0; i < this->num_matches; i++) {
         ir_variable *producer_var = this->matches[i].producer_var;
         ir_variable *consumer_var = this->matches[i].consumer_var;
         unsigned generic_location = this->matches[i].generic_location;
         unsigned slot = generic_location / 4;

         if (pack_loc[slot] || !producer_var || !consumer_var)
            continue;

         const glsl_type *type =
            get_varying_type(producer_var, producer_stage);
         bool type_match = true;
         for (unsigned j = 0; j < 4; j++) {
            if (loc_type[slot][j]) {
               if (type->base_type != loc_type[slot][j]->base_type)
                  type_match = false;
            }
         }

         if (type_match) {
            producer_var->data.explicit_location = 1;
            consumer_var->data.explicit_location = 1;
            producer_var->data.explicit_component = 1;
            consumer_var->data.explicit_component = 1;
         }
      }
   }
}


/**
 * Compute the "packing class" of the given varying.  This is an unsigned
 * integer with the property that two variables in the same packing class can
 * be safely backed into the same vec4.
 */
unsigned
varying_matches::compute_packing_class(const ir_variable *var)
{
   /* Without help from the back-end, there is no way to pack together
    * variables with different interpolation types, because
    * lower_packed_varyings must choose exactly one interpolation type for
    * each packed varying it creates.
    *
    * However, we can safely pack together floats, ints, and uints, because:
    *
    * - varyings of base type "int" and "uint" must use the "flat"
    *   interpolation type, which can only occur in GLSL 1.30 and above.
    *
    * - On platforms that support GLSL 1.30 and above, lower_packed_varyings
    *   can store flat floats as ints without losing any information (using
    *   the ir_unop_bitcast_* opcodes).
    *
    * Therefore, the packing class depends only on the interpolation type.
    */
   const unsigned interp = var->is_interpolation_flat()
      ? unsigned(INTERP_MODE_FLAT) : var->data.interpolation;

   assert(interp < (1 << 3));

   const unsigned packing_class = (interp << 0) |
                                  (var->data.centroid << 3) |
                                  (var->data.sample << 4) |
                                  (var->data.patch << 5) |
                                  (var->data.must_be_shader_input << 6);

   return packing_class;
}


/**
 * Compute the "packing order" of the given varying.  This is a sort key we
 * use to determine when to attempt to pack the given varying relative to
 * other varyings in the same packing class.
 */
varying_matches::packing_order_enum
varying_matches::compute_packing_order(const ir_variable *var)
{
   const glsl_type *element_type = var->type;

   while (element_type->is_array()) {
      element_type = element_type->fields.array;
   }

   switch (element_type->component_slots() % 4) {
   case 1: return PACKING_ORDER_SCALAR;
   case 2: return PACKING_ORDER_VEC2;
   case 3: return PACKING_ORDER_VEC3;
   case 0: return PACKING_ORDER_VEC4;
   default:
      assert(!"Unexpected value of vector_elements");
      return PACKING_ORDER_VEC4;
   }
}


/**
 * Comparison function passed to qsort() to sort varyings by packing_class and
 * then by packing_order.
 */
int
varying_matches::match_comparator(const void *x_generic, const void *y_generic)
{
   const match *x = (const match *) x_generic;
   const match *y = (const match *) y_generic;

   if (x->packing_class != y->packing_class)
      return x->packing_class - y->packing_class;
   return x->packing_order - y->packing_order;
}


/**
 * Comparison function passed to qsort() to sort varyings used only by
 * transform feedback when packing of other varyings is disabled.
 */
int
varying_matches::xfb_comparator(const void *x_generic, const void *y_generic)
{
   const match *x = (const match *) x_generic;

   if (x->producer_var != NULL && x->producer_var->data.is_xfb_only)
      return match_comparator(x_generic, y_generic);

   /* FIXME: When the comparator returns 0 it means the elements being
    * compared are equivalent. However the qsort documentation says:
    *
    *    "The order of equivalent elements is undefined."
    *
    * In practice the sort ends up reversing the order of the varyings which
    * means locations are also assigned in this reversed order and happens to
    * be what we want. This is also whats happening in
    * varying_matches::match_comparator().
    */
   return 0;
}


/**
 * Comparison function passed to qsort() to sort varyings NOT used by
 * transform feedback when packing of xfb varyings is disabled.
 */
int
varying_matches::not_xfb_comparator(const void *x_generic, const void *y_generic)
{
   const match *x = (const match *) x_generic;

   if (x->producer_var != NULL && !x->producer_var->data.is_xfb)
      return match_comparator(x_generic, y_generic);

   /* FIXME: When the comparator returns 0 it means the elements being
    * compared are equivalent. However the qsort documentation says:
    *
    *    "The order of equivalent elements is undefined."
    *
    * In practice the sort ends up reversing the order of the varyings which
    * means locations are also assigned in this reversed order and happens to
    * be what we want. This is also whats happening in
    * varying_matches::match_comparator().
    */
   return 0;
}


/**
 * Is the given variable a varying variable to be counted against the
 * limit in ctx->Const.MaxVarying?
 * This includes variables such as texcoords, colors and generic
 * varyings, but excludes variables such as gl_FrontFacing and gl_FragCoord.
 */
static bool
var_counts_against_varying_limit(gl_shader_stage stage, const ir_variable *var)
{
   /* Only fragment shaders will take a varying variable as an input */
   if (stage == MESA_SHADER_FRAGMENT &&
       var->data.mode == ir_var_shader_in) {
      switch (var->data.location) {
      case VARYING_SLOT_POS:
      case VARYING_SLOT_FACE:
      case VARYING_SLOT_PNTC:
         return false;
      default:
         return true;
      }
   }
   return false;
}


/**
 * Visitor class that generates tfeedback_candidate structs describing all
 * possible targets of transform feedback.
 *
 * tfeedback_candidate structs are stored in the hash table
 * tfeedback_candidates, which is passed to the constructor.  This hash table
 * maps varying names to instances of the tfeedback_candidate struct.
 */
class tfeedback_candidate_generator : public program_resource_visitor
{
public:
   tfeedback_candidate_generator(void *mem_ctx,
                                 hash_table *tfeedback_candidates,
                                 gl_shader_stage stage)
      : mem_ctx(mem_ctx),
        tfeedback_candidates(tfeedback_candidates),
        stage(stage),
        toplevel_var(NULL),
        varying_floats(0)
   {
   }

   void process(ir_variable *var)
   {
      /* All named varying interface blocks should be flattened by now */
      assert(!var->is_interface_instance());
      assert(var->data.mode == ir_var_shader_out);

      this->toplevel_var = var;
      this->varying_floats = 0;
      const glsl_type *t =
         var->data.from_named_ifc_block ? var->get_interface_type() : var->type;
      if (!var->data.patch && stage == MESA_SHADER_TESS_CTRL) {
         assert(t->is_array());
         t = t->fields.array;
      }
      program_resource_visitor::process(var, t, false);
   }

private:
   virtual void visit_field(const glsl_type *type, const char *name,
                            bool /* row_major */,
                            const glsl_type * /* record_type */,
                            const enum glsl_interface_packing,
                            bool /* last_field */)
   {
      assert(!type->without_array()->is_struct());
      assert(!type->without_array()->is_interface());

      tfeedback_candidate *candidate
         = rzalloc(this->mem_ctx, tfeedback_candidate);
      candidate->toplevel_var = this->toplevel_var;
      candidate->type = type;
      candidate->offset = this->varying_floats;
      _mesa_hash_table_insert(this->tfeedback_candidates,
                              ralloc_strdup(this->mem_ctx, name),
                              candidate);
      this->varying_floats += type->component_slots();
   }

   /**
    * Memory context used to allocate hash table keys and values.
    */
   void * const mem_ctx;

   /**
    * Hash table in which tfeedback_candidate objects should be stored.
    */
   hash_table * const tfeedback_candidates;

   gl_shader_stage stage;

   /**
    * Pointer to the toplevel variable that is being traversed.
    */
   ir_variable *toplevel_var;

   /**
    * Total number of varying floats that have been visited so far.  This is
    * used to determine the offset to each varying within the toplevel
    * variable.
    */
   unsigned varying_floats;
};


namespace linker {

void
populate_consumer_input_sets(void *mem_ctx, exec_list *ir,
                             hash_table *consumer_inputs,
                             hash_table *consumer_interface_inputs,
                             ir_variable *consumer_inputs_with_locations[VARYING_SLOT_TESS_MAX])
{
   memset(consumer_inputs_with_locations,
          0,
          sizeof(consumer_inputs_with_locations[0]) * VARYING_SLOT_TESS_MAX);

   foreach_in_list(ir_instruction, node, ir) {
      ir_variable *const input_var = node->as_variable();

      if (input_var != NULL && input_var->data.mode == ir_var_shader_in) {
         /* All interface blocks should have been lowered by this point */
         assert(!input_var->type->is_interface());

         if (input_var->data.explicit_location) {
            /* assign_varying_locations only cares about finding the
             * ir_variable at the start of a contiguous location block.
             *
             *     - For !producer, consumer_inputs_with_locations isn't used.
             *
             *     - For !consumer, consumer_inputs_with_locations is empty.
             *
             * For consumer && producer, if you were trying to set some
             * ir_variable to the middle of a location block on the other side
             * of producer/consumer, cross_validate_outputs_to_inputs() should
             * be link-erroring due to either type mismatch or location
             * overlaps.  If the variables do match up, then they've got a
             * matching data.location and you only looked at
             * consumer_inputs_with_locations[var->data.location], not any
             * following entries for the array/structure.
             */
            consumer_inputs_with_locations[input_var->data.location] =
               input_var;
         } else if (input_var->get_interface_type() != NULL) {
            char *const iface_field_name =
               ralloc_asprintf(mem_ctx, "%s.%s",
                  input_var->get_interface_type()->without_array()->name,
                  input_var->name);
            _mesa_hash_table_insert(consumer_interface_inputs,
                                    iface_field_name, input_var);
         } else {
            _mesa_hash_table_insert(consumer_inputs,
                                    ralloc_strdup(mem_ctx, input_var->name),
                                    input_var);
         }
      }
   }
}

/**
 * Find a variable from the consumer that "matches" the specified variable
 *
 * This function only finds inputs with names that match.  There is no
 * validation (here) that the types, etc. are compatible.
 */
ir_variable *
get_matching_input(void *mem_ctx,
                   const ir_variable *output_var,
                   hash_table *consumer_inputs,
                   hash_table *consumer_interface_inputs,
                   ir_variable *consumer_inputs_with_locations[VARYING_SLOT_TESS_MAX])
{
   ir_variable *input_var;

   if (output_var->data.explicit_location) {
      input_var = consumer_inputs_with_locations[output_var->data.location];
   } else if (output_var->get_interface_type() != NULL) {
      char *const iface_field_name =
         ralloc_asprintf(mem_ctx, "%s.%s",
            output_var->get_interface_type()->without_array()->name,
            output_var->name);
      hash_entry *entry = _mesa_hash_table_search(consumer_interface_inputs, iface_field_name);
      input_var = entry ? (ir_variable *) entry->data : NULL;
   } else {
      hash_entry *entry = _mesa_hash_table_search(consumer_inputs, output_var->name);
      input_var = entry ? (ir_variable *) entry->data : NULL;
   }

   return (input_var == NULL || input_var->data.mode != ir_var_shader_in)
      ? NULL : input_var;
}

}

static int
io_variable_cmp(const void *_a, const void *_b)
{
   const ir_variable *const a = *(const ir_variable **) _a;
   const ir_variable *const b = *(const ir_variable **) _b;

   if (a->data.explicit_location && b->data.explicit_location)
      return b->data.location - a->data.location;

   if (a->data.explicit_location && !b->data.explicit_location)
      return 1;

   if (!a->data.explicit_location && b->data.explicit_location)
      return -1;

   return -strcmp(a->name, b->name);
}

/**
 * Sort the shader IO variables into canonical order
 */
static void
canonicalize_shader_io(exec_list *ir, enum ir_variable_mode io_mode)
{
   ir_variable *var_table[MAX_PROGRAM_OUTPUTS * 4];
   unsigned num_variables = 0;

   foreach_in_list(ir_instruction, node, ir) {
      ir_variable *const var = node->as_variable();

      if (var == NULL || var->data.mode != io_mode)
         continue;

      /* If we have already encountered more I/O variables that could
       * successfully link, bail.
       */
      if (num_variables == ARRAY_SIZE(var_table))
         return;

      var_table[num_variables++] = var;
   }

   if (num_variables == 0)
      return;

   /* Sort the list in reverse order (io_variable_cmp handles this).  Later
    * we're going to push the variables on to the IR list as a stack, so we
    * want the last variable (in canonical order) to be first in the list.
    */
   qsort(var_table, num_variables, sizeof(var_table[0]), io_variable_cmp);

   /* Remove the variable from it's current location in the IR, and put it at
    * the front.
    */
   for (unsigned i = 0; i < num_variables; i++) {
      var_table[i]->remove();
      ir->push_head(var_table[i]);
   }
}

/**
 * Generate a bitfield map of the explicit locations for shader varyings.
 *
 * Note: For Tessellation shaders we are sitting right on the limits of the
 * 64 bit map. Per-vertex and per-patch both have separate location domains
 * with a max of MAX_VARYING.
 */
static uint64_t
reserved_varying_slot(struct gl_linked_shader *stage,
                      ir_variable_mode io_mode)
{
   assert(io_mode == ir_var_shader_in || io_mode == ir_var_shader_out);
   /* Avoid an overflow of the returned value */
   assert(MAX_VARYINGS_INCL_PATCH <= 64);

   uint64_t slots = 0;
   int var_slot;

   if (!stage)
      return slots;

   foreach_in_list(ir_instruction, node, stage->ir) {
      ir_variable *const var = node->as_variable();

      if (var == NULL || var->data.mode != io_mode ||
          !var->data.explicit_location ||
          var->data.location < VARYING_SLOT_VAR0)
         continue;

      var_slot = var->data.location - VARYING_SLOT_VAR0;

      unsigned num_elements = get_varying_type(var, stage->Stage)
         ->count_attribute_slots(io_mode == ir_var_shader_in &&
                                 stage->Stage == MESA_SHADER_VERTEX);
      for (unsigned i = 0; i < num_elements; i++) {
         if (var_slot >= 0 && var_slot < MAX_VARYINGS_INCL_PATCH)
            slots |= UINT64_C(1) << var_slot;
         var_slot += 1;
      }
   }

   return slots;
}


/**
 * Assign locations for all variables that are produced in one pipeline stage
 * (the "producer") and consumed in the next stage (the "consumer").
 *
 * Variables produced by the producer may also be consumed by transform
 * feedback.
 *
 * \param num_tfeedback_decls is the number of declarations indicating
 *        variables that may be consumed by transform feedback.
 *
 * \param tfeedback_decls is a pointer to an array of tfeedback_decl objects
 *        representing the result of parsing the strings passed to
 *        glTransformFeedbackVaryings().  assign_location() will be called for
 *        each of these objects that matches one of the outputs of the
 *        producer.
 *
 * When num_tfeedback_decls is nonzero, it is permissible for the consumer to
 * be NULL.  In this case, varying locations are assigned solely based on the
 * requirements of transform feedback.
 */
static bool
assign_varying_locations(struct gl_context *ctx,
                         void *mem_ctx,
                         struct gl_shader_program *prog,
                         gl_linked_shader *producer,
                         gl_linked_shader *consumer,
                         unsigned num_tfeedback_decls,
                         tfeedback_decl *tfeedback_decls,
                         const uint64_t reserved_slots)
{
   /* Tessellation shaders treat inputs and outputs as shared memory and can
    * access inputs and outputs of other invocations.
    * Therefore, they can't be lowered to temps easily (and definitely not
    * efficiently).
    */
   bool unpackable_tess =
      (consumer && consumer->Stage == MESA_SHADER_TESS_EVAL) ||
      (consumer && consumer->Stage == MESA_SHADER_TESS_CTRL) ||
      (producer && producer->Stage == MESA_SHADER_TESS_CTRL);

   /* Transform feedback code assumes varying arrays are packed, so if the
    * driver has disabled varying packing, make sure to at least enable
    * packing required by transform feedback. See below for exception.
    */
   bool xfb_enabled =
      ctx->Extensions.EXT_transform_feedback && !unpackable_tess;

   /* Some drivers actually requires packing to be explicitly disabled
    * for varyings used by transform feedback.
    */
   bool disable_xfb_packing =
      ctx->Const.DisableTransformFeedbackPacking;

   /* Disable packing on outward facing interfaces for SSO because in ES we
    * need to retain the unpacked varying information for draw time
    * validation.
    *
    * Packing is still enabled on individual arrays, structs, and matrices as
    * these are required by the transform feedback code and it is still safe
    * to do so. We also enable packing when a varying is only used for
    * transform feedback and its not a SSO.
    */
   bool disable_varying_packing =
      ctx->Const.DisableVaryingPacking || unpackable_tess;
   if (prog->SeparateShader && (producer == NULL || consumer == NULL))
      disable_varying_packing = true;

   varying_matches matches(disable_varying_packing,
                           disable_xfb_packing,
                           xfb_enabled,
                           ctx->Extensions.ARB_enhanced_layouts,
                           producer ? producer->Stage : MESA_SHADER_NONE,
                           consumer ? consumer->Stage : MESA_SHADER_NONE);
   void *hash_table_ctx = ralloc_context(NULL);
   hash_table *tfeedback_candidates =
         _mesa_hash_table_create(hash_table_ctx, _mesa_hash_string,
                                 _mesa_key_string_equal);
   hash_table *consumer_inputs =
         _mesa_hash_table_create(hash_table_ctx, _mesa_hash_string,
                                 _mesa_key_string_equal);
   hash_table *consumer_interface_inputs =
         _mesa_hash_table_create(hash_table_ctx, _mesa_hash_string,
                                 _mesa_key_string_equal);
   ir_variable *consumer_inputs_with_locations[VARYING_SLOT_TESS_MAX] = {
      NULL,
   };

   unsigned consumer_vertices = 0;
   if (consumer && consumer->Stage == MESA_SHADER_GEOMETRY)
      consumer_vertices = prog->Geom.VerticesIn;

   /* Operate in a total of four passes.
    *
    * 1. Sort inputs / outputs into a canonical order.  This is necessary so
    *    that inputs / outputs of separable shaders will be assigned
    *    predictable locations regardless of the order in which declarations
    *    appeared in the shader source.
    *
    * 2. Assign locations for any matching inputs and outputs.
    *
    * 3. Mark output variables in the producer that do not have locations as
    *    not being outputs.  This lets the optimizer eliminate them.
    *
    * 4. Mark input variables in the consumer that do not have locations as
    *    not being inputs.  This lets the optimizer eliminate them.
    */
   if (consumer)
      canonicalize_shader_io(consumer->ir, ir_var_shader_in);

   if (producer)
      canonicalize_shader_io(producer->ir, ir_var_shader_out);

   if (consumer)
      linker::populate_consumer_input_sets(mem_ctx, consumer->ir,
                                           consumer_inputs,
                                           consumer_interface_inputs,
                                           consumer_inputs_with_locations);

   if (producer) {
      foreach_in_list(ir_instruction, node, producer->ir) {
         ir_variable *const output_var = node->as_variable();

         if (output_var == NULL || output_var->data.mode != ir_var_shader_out)
            continue;

         /* Only geometry shaders can use non-zero streams */
         assert(output_var->data.stream == 0 ||
                (output_var->data.stream < MAX_VERTEX_STREAMS &&
                 producer->Stage == MESA_SHADER_GEOMETRY));

         if (num_tfeedback_decls > 0) {
            tfeedback_candidate_generator g(mem_ctx, tfeedback_candidates, producer->Stage);
            /* From OpenGL 4.6 (Core Profile) spec, section 11.1.2.1
             * ("Vertex Shader Variables / Output Variables")
             *
             * "Each program object can specify a set of output variables from
             * one shader to be recorded in transform feedback mode (see
             * section 13.3). The variables that can be recorded are those
             * emitted by the first active shader, in order, from the
             * following list:
             *
             *  * geometry shader
             *  * tessellation evaluation shader
             *  * tessellation control shader
             *  * vertex shader"
             *
             * But on OpenGL ES 3.2, section 11.1.2.1 ("Vertex Shader
             * Variables / Output Variables") tessellation control shader is
             * not included in the stages list.
             */
            if (!prog->IsES || producer->Stage != MESA_SHADER_TESS_CTRL) {
               g.process(output_var);
            }
         }

         ir_variable *const input_var =
            linker::get_matching_input(mem_ctx, output_var, consumer_inputs,
                                       consumer_interface_inputs,
                                       consumer_inputs_with_locations);

         /* If a matching input variable was found, add this output (and the
          * input) to the set.  If this is a separable program and there is no
          * consumer stage, add the output.
          *
          * Always add TCS outputs. They are shared by all invocations
          * within a patch and can be used as shared memory.
          */
         if (input_var || (prog->SeparateShader && consumer == NULL) ||
             producer->Stage == MESA_SHADER_TESS_CTRL) {
            matches.record(output_var, input_var);
         }

         /* Only stream 0 outputs can be consumed in the next stage */
         if (input_var && output_var->data.stream != 0) {
            linker_error(prog, "output %s is assigned to stream=%d but "
                         "is linked to an input, which requires stream=0",
                         output_var->name, output_var->data.stream);
            ralloc_free(hash_table_ctx);
            return false;
         }
      }
   } else {
      /* If there's no producer stage, then this must be a separable program.
       * For example, we may have a program that has just a fragment shader.
       * Later this program will be used with some arbitrary vertex (or
       * geometry) shader program.  This means that locations must be assigned
       * for all the inputs.
       */
      foreach_in_list(ir_instruction, node, consumer->ir) {
         ir_variable *const input_var = node->as_variable();
         if (input_var && input_var->data.mode == ir_var_shader_in) {
            matches.record(NULL, input_var);
         }
      }
   }

   for (unsigned i = 0; i < num_tfeedback_decls; ++i) {
      if (!tfeedback_decls[i].is_varying())
         continue;

      const tfeedback_candidate *matched_candidate
         = tfeedback_decls[i].find_candidate(prog, tfeedback_candidates);

      if (matched_candidate == NULL) {
         ralloc_free(hash_table_ctx);
         return false;
      }

      /* There are two situations where a new output varying is needed:
       *
       *  - If varying packing is disabled for xfb and the current declaration
       *    is not aligned within the top level varying (e.g. vec3_arr[1]).
       *
       *  - If a builtin variable needs to be copied to a new variable
       *    before its content is modified by another lowering pass (e.g.
       *    \c gl_Position is transformed by \c nir_lower_viewport_transform).
       */
      const unsigned dmul =
         matched_candidate->type->without_array()->is_64bit() ? 2 : 1;
      const bool lowered =
         (disable_xfb_packing &&
          !tfeedback_decls[i].is_aligned(dmul, matched_candidate->offset)) ||
         (matched_candidate->toplevel_var->data.explicit_location &&
          matched_candidate->toplevel_var->data.location < VARYING_SLOT_VAR0 &&
          (ctx->Const.ShaderCompilerOptions[producer->Stage].LowerBuiltinVariablesXfb &
              BITFIELD_BIT(matched_candidate->toplevel_var->data.location)));

      if (lowered) {
         ir_variable *new_var;
         tfeedback_candidate *new_candidate = NULL;

         new_var = lower_xfb_varying(mem_ctx, producer, tfeedback_decls[i].name());
         if (new_var == NULL) {
            ralloc_free(hash_table_ctx);
            return false;
         }

         /* Create new candidate and replace matched_candidate */
         new_candidate = rzalloc(mem_ctx, tfeedback_candidate);
         new_candidate->toplevel_var = new_var;
         new_candidate->toplevel_var->data.is_unmatched_generic_inout = 1;
         new_candidate->type = new_var->type;
         new_candidate->offset = 0;
         _mesa_hash_table_insert(tfeedback_candidates,
                                 ralloc_strdup(mem_ctx, new_var->name),
                                 new_candidate);

         tfeedback_decls[i].set_lowered_candidate(new_candidate);
         matched_candidate = new_candidate;
      }

      /* Mark as xfb varying */
      matched_candidate->toplevel_var->data.is_xfb = 1;

      /* Mark xfb varyings as always active */
      matched_candidate->toplevel_var->data.always_active_io = 1;

      /* Mark any corresponding inputs as always active also. We must do this
       * because we have a NIR pass that lowers vectors to scalars and another
       * that removes unused varyings.
       * We don't split varyings marked as always active because there is no
       * point in doing so. This means we need to mark both sides of the
       * interface as always active otherwise we will have a mismatch and
       * start removing things we shouldn't.
       */
      ir_variable *const input_var =
         linker::get_matching_input(mem_ctx, matched_candidate->toplevel_var,
                                    consumer_inputs,
                                    consumer_interface_inputs,
                                    consumer_inputs_with_locations);
      if (input_var) {
         input_var->data.is_xfb = 1;
         input_var->data.always_active_io = 1;
      }

      if (matched_candidate->toplevel_var->data.is_unmatched_generic_inout) {
         matched_candidate->toplevel_var->data.is_xfb_only = 1;
         matches.record(matched_candidate->toplevel_var, NULL);
      }
   }

   uint8_t components[MAX_VARYINGS_INCL_PATCH] = {0};
   const unsigned slots_used = matches.assign_locations(
         prog, components, reserved_slots);
   matches.store_locations();

   for (unsigned i = 0; i < num_tfeedback_decls; ++i) {
      if (tfeedback_decls[i].is_varying()) {
         if (!tfeedback_decls[i].assign_location(ctx, prog)) {
            ralloc_free(hash_table_ctx);
            return false;
         }
      }
   }
   ralloc_free(hash_table_ctx);

   if (consumer && producer) {
      foreach_in_list(ir_instruction, node, consumer->ir) {
         ir_variable *const var = node->as_variable();

         if (var && var->data.mode == ir_var_shader_in &&
             var->data.is_unmatched_generic_inout) {
            if (!prog->IsES && prog->data->Version <= 120) {
               /* On page 25 (page 31 of the PDF) of the GLSL 1.20 spec:
                *
                *     Only those varying variables used (i.e. read) in
                *     the fragment shader executable must be written to
                *     by the vertex shader executable; declaring
                *     superfluous varying variables in a vertex shader is
                *     permissible.
                *
                * We interpret this text as meaning that the VS must
                * write the variable for the FS to read it.  See
                * "glsl1-varying read but not written" in piglit.
                */
               linker_error(prog, "%s shader varying %s not written "
                            "by %s shader\n.",
                            _mesa_shader_stage_to_string(consumer->Stage),
                            var->name,
                            _mesa_shader_stage_to_string(producer->Stage));
            } else {
               linker_warning(prog, "%s shader varying %s not written "
                              "by %s shader\n.",
                              _mesa_shader_stage_to_string(consumer->Stage),
                              var->name,
                              _mesa_shader_stage_to_string(producer->Stage));
            }
         }
      }

      /* Now that validation is done its safe to remove unused varyings. As
       * we have both a producer and consumer its safe to remove unused
       * varyings even if the program is a SSO because the stages are being
       * linked together i.e. we have a multi-stage SSO.
       */
      remove_unused_shader_inputs_and_outputs(false, producer,
                                              ir_var_shader_out);
      remove_unused_shader_inputs_and_outputs(false, consumer,
                                              ir_var_shader_in);
   }

   if (producer) {
      lower_packed_varyings(mem_ctx, slots_used, components, ir_var_shader_out,
                            0, producer, disable_varying_packing,
                            disable_xfb_packing, xfb_enabled);
   }

   if (consumer) {
      lower_packed_varyings(mem_ctx, slots_used, components, ir_var_shader_in,
                            consumer_vertices, consumer, disable_varying_packing,
                            disable_xfb_packing, xfb_enabled);
   }

   return true;
}

static bool
check_against_output_limit(struct gl_context *ctx,
                           struct gl_shader_program *prog,
                           gl_linked_shader *producer,
                           unsigned num_explicit_locations)
{
   unsigned output_vectors = num_explicit_locations;

   foreach_in_list(ir_instruction, node, producer->ir) {
      ir_variable *const var = node->as_variable();

      if (var && !var->data.explicit_location &&
          var->data.mode == ir_var_shader_out &&
          var_counts_against_varying_limit(producer->Stage, var)) {
         /* outputs for fragment shader can't be doubles */
         output_vectors += var->type->count_attribute_slots(false);
      }
   }

   assert(producer->Stage != MESA_SHADER_FRAGMENT);
   unsigned max_output_components =
      ctx->Const.Program[producer->Stage].MaxOutputComponents;

   const unsigned output_components = output_vectors * 4;
   if (output_components > max_output_components) {
      if (ctx->API == API_OPENGLES2 || prog->IsES)
         linker_error(prog, "%s shader uses too many output vectors "
                      "(%u > %u)\n",
                      _mesa_shader_stage_to_string(producer->Stage),
                      output_vectors,
                      max_output_components / 4);
      else
         linker_error(prog, "%s shader uses too many output components "
                      "(%u > %u)\n",
                      _mesa_shader_stage_to_string(producer->Stage),
                      output_components,
                      max_output_components);

      return false;
   }

   return true;
}

static bool
check_against_input_limit(struct gl_context *ctx,
                          struct gl_shader_program *prog,
                          gl_linked_shader *consumer,
                          unsigned num_explicit_locations)
{
   unsigned input_vectors = num_explicit_locations;

   foreach_in_list(ir_instruction, node, consumer->ir) {
      ir_variable *const var = node->as_variable();

      if (var && !var->data.explicit_location &&
          var->data.mode == ir_var_shader_in &&
          var_counts_against_varying_limit(consumer->Stage, var)) {
         /* vertex inputs aren't varying counted */
         input_vectors += var->type->count_attribute_slots(false);
      }
   }

   assert(consumer->Stage != MESA_SHADER_VERTEX);
   unsigned max_input_components =
      ctx->Const.Program[consumer->Stage].MaxInputComponents;

   const unsigned input_components = input_vectors * 4;
   if (input_components > max_input_components) {
      if (ctx->API == API_OPENGLES2 || prog->IsES)
         linker_error(prog, "%s shader uses too many input vectors "
                      "(%u > %u)\n",
                      _mesa_shader_stage_to_string(consumer->Stage),
                      input_vectors,
                      max_input_components / 4);
      else
         linker_error(prog, "%s shader uses too many input components "
                      "(%u > %u)\n",
                      _mesa_shader_stage_to_string(consumer->Stage),
                      input_components,
                      max_input_components);

      return false;
   }

   return true;
}

bool
link_varyings(struct gl_shader_program *prog, unsigned first, unsigned last,
              struct gl_context *ctx, void *mem_ctx)
{
   bool has_xfb_qualifiers = false;
   unsigned num_tfeedback_decls = 0;
   char **varying_names = NULL;
   tfeedback_decl *tfeedback_decls = NULL;

   /* From the ARB_enhanced_layouts spec:
    *
    *    "If the shader used to record output variables for transform feedback
    *    varyings uses the "xfb_buffer", "xfb_offset", or "xfb_stride" layout
    *    qualifiers, the values specified by TransformFeedbackVaryings are
    *    ignored, and the set of variables captured for transform feedback is
    *    instead derived from the specified layout qualifiers."
    */
   for (int i = MESA_SHADER_FRAGMENT - 1; i >= 0; i--) {
      /* Find last stage before fragment shader */
      if (prog->_LinkedShaders[i]) {
         has_xfb_qualifiers =
            process_xfb_layout_qualifiers(mem_ctx, prog->_LinkedShaders[i],
                                          prog, &num_tfeedback_decls,
                                          &varying_names);
         break;
      }
   }

   if (!has_xfb_qualifiers) {
      num_tfeedback_decls = prog->TransformFeedback.NumVarying;
      varying_names = prog->TransformFeedback.VaryingNames;
   }

   if (num_tfeedback_decls != 0) {
      /* From GL_EXT_transform_feedback:
       *   A program will fail to link if:
       *
       *   * the <count> specified by TransformFeedbackVaryingsEXT is
       *     non-zero, but the program object has no vertex or geometry
       *     shader;
       */
      if (first >= MESA_SHADER_FRAGMENT) {
         linker_error(prog, "Transform feedback varyings specified, but "
                      "no vertex, tessellation, or geometry shader is "
                      "present.\n");
         return false;
      }

      tfeedback_decls = rzalloc_array(mem_ctx, tfeedback_decl,
                                      num_tfeedback_decls);
      if (!parse_tfeedback_decls(ctx, prog, mem_ctx, num_tfeedback_decls,
                                 varying_names, tfeedback_decls))
         return false;
   }

   /* If there is no fragment shader we need to set transform feedback.
    *
    * For SSO we also need to assign output locations.  We assign them here
    * because we need to do it for both single stage programs and multi stage
    * programs.
    */
   if (last < MESA_SHADER_FRAGMENT &&
       (num_tfeedback_decls != 0 || prog->SeparateShader)) {
      const uint64_t reserved_out_slots =
         reserved_varying_slot(prog->_LinkedShaders[last], ir_var_shader_out);
      if (!assign_varying_locations(ctx, mem_ctx, prog,
                                    prog->_LinkedShaders[last], NULL,
                                    num_tfeedback_decls, tfeedback_decls,
                                    reserved_out_slots))
         return false;
   }

   if (last <= MESA_SHADER_FRAGMENT) {
      /* Remove unused varyings from the first/last stage unless SSO */
      remove_unused_shader_inputs_and_outputs(prog->SeparateShader,
                                              prog->_LinkedShaders[first],
                                              ir_var_shader_in);
      remove_unused_shader_inputs_and_outputs(prog->SeparateShader,
                                              prog->_LinkedShaders[last],
                                              ir_var_shader_out);

      /* If the program is made up of only a single stage */
      if (first == last) {
         gl_linked_shader *const sh = prog->_LinkedShaders[last];

         do_dead_builtin_varyings(ctx, NULL, sh, 0, NULL);
         do_dead_builtin_varyings(ctx, sh, NULL, num_tfeedback_decls,
                                  tfeedback_decls);

         if (prog->SeparateShader) {
            const uint64_t reserved_slots =
               reserved_varying_slot(sh, ir_var_shader_in);

            /* Assign input locations for SSO, output locations are already
             * assigned.
             */
            if (!assign_varying_locations(ctx, mem_ctx, prog,
                                          NULL /* producer */,
                                          sh /* consumer */,
                                          0 /* num_tfeedback_decls */,
                                          NULL /* tfeedback_decls */,
                                          reserved_slots))
               return false;
         }
      } else {
         /* Linking the stages in the opposite order (from fragment to vertex)
          * ensures that inter-shader outputs written to in an earlier stage
          * are eliminated if they are (transitively) not used in a later
          * stage.
          */
         int next = last;
         for (int i = next - 1; i >= 0; i--) {
            if (prog->_LinkedShaders[i] == NULL && i != 0)
               continue;

            gl_linked_shader *const sh_i = prog->_LinkedShaders[i];
            gl_linked_shader *const sh_next = prog->_LinkedShaders[next];

            const uint64_t reserved_out_slots =
               reserved_varying_slot(sh_i, ir_var_shader_out);
            const uint64_t reserved_in_slots =
               reserved_varying_slot(sh_next, ir_var_shader_in);

            do_dead_builtin_varyings(ctx, sh_i, sh_next,
                      next == MESA_SHADER_FRAGMENT ? num_tfeedback_decls : 0,
                      tfeedback_decls);

            if (!assign_varying_locations(ctx, mem_ctx, prog, sh_i, sh_next,
                      next == MESA_SHADER_FRAGMENT ? num_tfeedback_decls : 0,
                      tfeedback_decls,
                      reserved_out_slots | reserved_in_slots))
               return false;

            /* This must be done after all dead varyings are eliminated. */
            if (sh_i != NULL) {
               unsigned slots_used = util_bitcount64(reserved_out_slots);
               if (!check_against_output_limit(ctx, prog, sh_i, slots_used)) {
                  return false;
               }
            }

            unsigned slots_used = util_bitcount64(reserved_in_slots);
            if (!check_against_input_limit(ctx, prog, sh_next, slots_used))
               return false;

            next = i;
         }
      }
   }

   if (!store_tfeedback_info(ctx, prog, num_tfeedback_decls, tfeedback_decls,
                             has_xfb_qualifiers, mem_ctx))
      return false;

   return true;
}
