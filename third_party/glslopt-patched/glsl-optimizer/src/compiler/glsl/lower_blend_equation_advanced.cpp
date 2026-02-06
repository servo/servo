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

#include "ir.h"
#include "ir_builder.h"
#include "ir_optimization.h"
#include "ir_hierarchical_visitor.h"
#include "program/prog_instruction.h"
#include "program/prog_statevars.h"
#include "util/bitscan.h"
#include "builtin_functions.h"
#include "main/mtypes.h"

using namespace ir_builder;

#define imm1(x) new(mem_ctx) ir_constant((float) (x), 1)
#define imm3(x) new(mem_ctx) ir_constant((float) (x), 3)

static ir_rvalue *
blend_multiply(ir_variable *src, ir_variable *dst)
{
   /* f(Cs,Cd) = Cs*Cd */
   return mul(src, dst);
}

static ir_rvalue *
blend_screen(ir_variable *src, ir_variable *dst)
{
   /* f(Cs,Cd) = Cs+Cd-Cs*Cd */
   return sub(add(src, dst), mul(src, dst));
}

static ir_rvalue *
blend_overlay(ir_variable *src, ir_variable *dst)
{
   void *mem_ctx = ralloc_parent(src);

   /* f(Cs,Cd) = 2*Cs*Cd, if Cd <= 0.5
    *            1-2*(1-Cs)*(1-Cd), otherwise
    */
   ir_rvalue *rule_1 = mul(imm3(2), mul(src, dst));
   ir_rvalue *rule_2 =
      sub(imm3(1), mul(imm3(2), mul(sub(imm3(1), src), sub(imm3(1), dst))));
   return csel(lequal(dst, imm3(0.5f)), rule_1, rule_2);
}

static ir_rvalue *
blend_darken(ir_variable *src, ir_variable *dst)
{
   /* f(Cs,Cd) = min(Cs,Cd) */
   return min2(src, dst);
}

static ir_rvalue *
blend_lighten(ir_variable *src, ir_variable *dst)
{
   /* f(Cs,Cd) = max(Cs,Cd) */
   return max2(src, dst);
}

static ir_rvalue *
blend_colordodge(ir_variable *src, ir_variable *dst)
{
   void *mem_ctx = ralloc_parent(src);

   /* f(Cs,Cd) =
    *   0, if Cd <= 0
    *   min(1,Cd/(1-Cs)), if Cd > 0 and Cs < 1
    *   1, if Cd > 0 and Cs >= 1
    */
   return csel(lequal(dst, imm3(0)), imm3(0),
               csel(gequal(src, imm3(1)), imm3(1),
                    min2(imm3(1), div(dst, sub(imm3(1), src)))));
}

static ir_rvalue *
blend_colorburn(ir_variable *src, ir_variable *dst)
{
   void *mem_ctx = ralloc_parent(src);

   /* f(Cs,Cd) =
    *   1, if Cd >= 1
    *   1 - min(1,(1-Cd)/Cs), if Cd < 1 and Cs > 0
    *   0, if Cd < 1 and Cs <= 0
    */
   return csel(gequal(dst, imm3(1)), imm3(1),
               csel(lequal(src, imm3(0)), imm3(0),
                    sub(imm3(1), min2(imm3(1), div(sub(imm3(1), dst), src)))));
}

static ir_rvalue *
blend_hardlight(ir_variable *src, ir_variable *dst)
{
   void *mem_ctx = ralloc_parent(src);

   /* f(Cs,Cd) = 2*Cs*Cd, if Cs <= 0.5
    *            1-2*(1-Cs)*(1-Cd), otherwise
    */
   ir_rvalue *rule_1 = mul(imm3(2), mul(src, dst));
   ir_rvalue *rule_2 =
      sub(imm3(1), mul(imm3(2), mul(sub(imm3(1), src), sub(imm3(1), dst))));
   return csel(lequal(src, imm3(0.5f)), rule_1, rule_2);
}

static ir_rvalue *
blend_softlight(ir_variable *src, ir_variable *dst)
{
   void *mem_ctx = ralloc_parent(src);

   /* f(Cs,Cd) =
    *   Cd-(1-2*Cs)*Cd*(1-Cd),
    *     if Cs <= 0.5
    *   Cd+(2*Cs-1)*Cd*((16*Cd-12)*Cd+3),
    *     if Cs > 0.5 and Cd <= 0.25
    *   Cd+(2*Cs-1)*(sqrt(Cd)-Cd),
    *     if Cs > 0.5 and Cd > 0.25
    *
    * We can simplify this to
    *
    * f(Cs,Cd) = Cd+(2*Cs-1)*g(Cs,Cd) where
    * g(Cs,Cd) = Cd*Cd-Cd             if Cs <= 0.5
    *            Cd*((16*Cd-12)*Cd+3) if Cs > 0.5 and Cd <= 0.25
    *            sqrt(Cd)-Cd,         otherwise
    */
   ir_rvalue *factor_1 = mul(dst, sub(imm3(1), dst));
   ir_rvalue *factor_2 =
      mul(dst, add(mul(sub(mul(imm3(16), dst), imm3(12)), dst), imm3(3)));
   ir_rvalue *factor_3 = sub(sqrt(dst), dst);
   ir_rvalue *factor = csel(lequal(src, imm3(0.5f)), factor_1,
                            csel(lequal(dst, imm3(0.25f)),
                                        factor_2, factor_3));
   return add(dst, mul(sub(mul(imm3(2), src), imm3(1)), factor));
}

static ir_rvalue *
blend_difference(ir_variable *src, ir_variable *dst)
{
   return abs(sub(dst, src));
}

static ir_rvalue *
blend_exclusion(ir_variable *src, ir_variable *dst)
{
   void *mem_ctx = ralloc_parent(src);

   return add(src, sub(dst, mul(imm3(2), mul(src, dst))));
}

/* Return the minimum of a vec3's components */
static ir_rvalue *
minv3(ir_variable *v)
{
   return min2(min2(swizzle_x(v), swizzle_y(v)), swizzle_z(v));
}

/* Return the maximum of a vec3's components */
static ir_rvalue *
maxv3(ir_variable *v)
{
   return max2(max2(swizzle_x(v), swizzle_y(v)), swizzle_z(v));
}

static ir_rvalue *
lumv3(ir_variable *c)
{
   ir_constant_data data;
   data.f[0] = 0.30;
   data.f[1] = 0.59;
   data.f[2] = 0.11;

   void *mem_ctx = ralloc_parent(c);

   /* dot(c, vec3(0.30, 0.59, 0.11)) */
   return dot(c, new(mem_ctx) ir_constant(glsl_type::vec3_type, &data));
}

static ir_rvalue *
satv3(ir_variable *c)
{
   return sub(maxv3(c), minv3(c));
}

/* Take the base RGB color <cbase> and override its luminosity with that
 * of the RGB color <clum>.
 *
 * This follows the equations given in the ES 3.2 (June 15th, 2016)
 * specification.  Revision 16 of GL_KHR_blend_equation_advanced and
 * revision 9 of GL_NV_blend_equation_advanced specify a different set
 * of equations.  Older revisions match ES 3.2's text, and dEQP expects
 * the ES 3.2 rules implemented here.
 */
static void
set_lum(ir_factory *f,
        ir_variable *color,
        ir_variable *cbase,
        ir_variable *clum)
{
   void *mem_ctx = f->mem_ctx;
   f->emit(assign(color, add(cbase, sub(lumv3(clum), lumv3(cbase)))));

   ir_variable *llum = f->make_temp(glsl_type::float_type, "__blend_lum");
   ir_variable *mincol = f->make_temp(glsl_type::float_type, "__blend_mincol");
   ir_variable *maxcol = f->make_temp(glsl_type::float_type, "__blend_maxcol");

   f->emit(assign(llum, lumv3(color)));
   f->emit(assign(mincol, minv3(color)));
   f->emit(assign(maxcol, maxv3(color)));

   f->emit(if_tree(less(mincol, imm1(0)),
                   assign(color, add(llum, div(mul(sub(color, llum), llum),
                                               sub(llum, mincol)))),
                   if_tree(greater(maxcol, imm1(1)),
                           assign(color, add(llum, div(mul(sub(color, llum),
                                                           sub(imm3(1), llum)),
                                                       sub(maxcol, llum)))))));

}

/* Take the base RGB color <cbase> and override its saturation with
 * that of the RGB color <csat>.  The override the luminosity of the
 * result with that of the RGB color <clum>.
 */
static void
set_lum_sat(ir_factory *f,
            ir_variable *color,
            ir_variable *cbase,
            ir_variable *csat,
            ir_variable *clum)
{
   void *mem_ctx = f->mem_ctx;

   ir_rvalue *minbase = minv3(cbase);
   ir_rvalue *ssat = satv3(csat);

   ir_variable *sbase = f->make_temp(glsl_type::float_type, "__blend_sbase");
   f->emit(assign(sbase, satv3(cbase)));

   /* Equivalent (modulo rounding errors) to setting the
    * smallest (R,G,B) component to 0, the largest to <ssat>,
    * and interpolating the "middle" component based on its
    * original value relative to the smallest/largest.
    */
   f->emit(if_tree(greater(sbase, imm1(0)),
                   assign(color, div(mul(sub(cbase, minbase), ssat), sbase)),
                   assign(color, imm3(0))));
   set_lum(f, color, color, clum);
}

static ir_rvalue *
is_mode(ir_variable *mode, enum gl_advanced_blend_mode q)
{
   return equal(mode, new(ralloc_parent(mode)) ir_constant(unsigned(q)));
}

static ir_variable *
calc_blend_result(ir_factory f,
                  ir_variable *mode,
                  ir_variable *fb,
                  ir_rvalue *blend_src,
                  GLbitfield blend_qualifiers)
{
   void *mem_ctx = f.mem_ctx;
   ir_variable *result = f.make_temp(glsl_type::vec4_type, "__blend_result");

   /* Save blend_src to a temporary so we can reference it multiple times. */
   ir_variable *src = f.make_temp(glsl_type::vec4_type, "__blend_src");
   f.emit(assign(src, blend_src));

   /* If we're not doing advanced blending, just write the original value. */
   ir_if *if_blending = new(mem_ctx) ir_if(is_mode(mode, BLEND_NONE));
   f.emit(if_blending);
   if_blending->then_instructions.push_tail(assign(result, src));

   f.instructions = &if_blending->else_instructions;

   /* (Rs', Gs', Bs') =
    *   (0, 0, 0),              if As == 0
    *   (Rs/As, Gs/As, Bs/As),  otherwise
    */
   ir_variable *src_rgb = f.make_temp(glsl_type::vec3_type, "__blend_src_rgb");
   ir_variable *src_alpha = f.make_temp(glsl_type::float_type, "__blend_src_a");

   /* (Rd', Gd', Bd') =
    *   (0, 0, 0),              if Ad == 0
    *   (Rd/Ad, Gd/Ad, Bd/Ad),  otherwise
    */
   ir_variable *dst_rgb = f.make_temp(glsl_type::vec3_type, "__blend_dst_rgb");
   ir_variable *dst_alpha = f.make_temp(glsl_type::float_type, "__blend_dst_a");

   f.emit(assign(dst_alpha, swizzle_w(fb)));
   f.emit(if_tree(equal(dst_alpha, imm1(0)),
                     assign(dst_rgb, imm3(0)),
                     assign(dst_rgb, csel(equal(swizzle_xyz(fb),
                                                swizzle(fb, SWIZZLE_WWWW, 3)),
                                          imm3(1),
                                          div(swizzle_xyz(fb), dst_alpha)))));

   f.emit(assign(src_alpha, swizzle_w(src)));
   f.emit(if_tree(equal(src_alpha, imm1(0)),
                     assign(src_rgb, imm3(0)),
                     assign(src_rgb, csel(equal(swizzle_xyz(src),
                                                swizzle(src, SWIZZLE_WWWW, 3)),
                                          imm3(1),
                                          div(swizzle_xyz(src), src_alpha)))));

   ir_variable *factor = f.make_temp(glsl_type::vec3_type, "__blend_factor");

   ir_factory casefactory = f;

   unsigned choices = blend_qualifiers;
   while (choices) {
      enum gl_advanced_blend_mode choice = (enum gl_advanced_blend_mode)
         (1u << u_bit_scan(&choices));

      ir_if *iff = new(mem_ctx) ir_if(is_mode(mode, choice));
      casefactory.emit(iff);
      casefactory.instructions = &iff->then_instructions;

      ir_rvalue *val = NULL;

      switch (choice) {
      case BLEND_MULTIPLY:
         val = blend_multiply(src_rgb, dst_rgb);
         break;
      case BLEND_SCREEN:
         val = blend_screen(src_rgb, dst_rgb);
         break;
      case BLEND_OVERLAY:
         val = blend_overlay(src_rgb, dst_rgb);
         break;
      case BLEND_DARKEN:
         val = blend_darken(src_rgb, dst_rgb);
         break;
      case BLEND_LIGHTEN:
         val = blend_lighten(src_rgb, dst_rgb);
         break;
      case BLEND_COLORDODGE:
         val = blend_colordodge(src_rgb, dst_rgb);
         break;
      case BLEND_COLORBURN:
         val = blend_colorburn(src_rgb, dst_rgb);
         break;
      case BLEND_HARDLIGHT:
         val = blend_hardlight(src_rgb, dst_rgb);
         break;
      case BLEND_SOFTLIGHT:
         val = blend_softlight(src_rgb, dst_rgb);
         break;
      case BLEND_DIFFERENCE:
         val = blend_difference(src_rgb, dst_rgb);
         break;
      case BLEND_EXCLUSION:
         val = blend_exclusion(src_rgb, dst_rgb);
         break;
      case BLEND_HSL_HUE:
         set_lum_sat(&casefactory, factor, src_rgb, dst_rgb, dst_rgb);
         break;
      case BLEND_HSL_SATURATION:
         set_lum_sat(&casefactory, factor, dst_rgb, src_rgb, dst_rgb);
         break;
      case BLEND_HSL_COLOR:
         set_lum(&casefactory, factor, src_rgb, dst_rgb);
         break;
      case BLEND_HSL_LUMINOSITY:
         set_lum(&casefactory, factor, dst_rgb, src_rgb);
         break;
      case BLEND_NONE:
      case BLEND_ALL:
         unreachable("not real cases");
      }

      if (val)
         casefactory.emit(assign(factor, val));

      casefactory.instructions = &iff->else_instructions;
   }

   /* p0(As,Ad) = As*Ad
    * p1(As,Ad) = As*(1-Ad)
    * p2(As,Ad) = Ad*(1-As)
    */
   ir_variable *p0 = f.make_temp(glsl_type::float_type, "__blend_p0");
   ir_variable *p1 = f.make_temp(glsl_type::float_type, "__blend_p1");
   ir_variable *p2 = f.make_temp(glsl_type::float_type, "__blend_p2");

   f.emit(assign(p0, mul(src_alpha, dst_alpha)));
   f.emit(assign(p1, mul(src_alpha, sub(imm1(1), dst_alpha))));
   f.emit(assign(p2, mul(dst_alpha, sub(imm1(1), src_alpha))));

   /* R = f(Rs',Rd')*p0(As,Ad) + Y*Rs'*p1(As,Ad) + Z*Rd'*p2(As,Ad)
    * G = f(Gs',Gd')*p0(As,Ad) + Y*Gs'*p1(As,Ad) + Z*Gd'*p2(As,Ad)
    * B = f(Bs',Bd')*p0(As,Ad) + Y*Bs'*p1(As,Ad) + Z*Bd'*p2(As,Ad)
    * A =          X*p0(As,Ad) +     Y*p1(As,Ad) +     Z*p2(As,Ad)
    *
    * <X, Y, Z> is always <1, 1, 1>, so we can ignore it.
    *
    * In vector form, this is:
    * RGB = factor * p0 + Cs * p1 + Cd * p2
    *   A = p0 + p1 + p2
    */
   f.emit(assign(result,
                 add(add(mul(factor, p0), mul(src_rgb, p1)), mul(dst_rgb, p2)),
                 WRITEMASK_XYZ));
   f.emit(assign(result, add(add(p0, p1), p2), WRITEMASK_W));

   return result;
}

/**
 * Dereference var, or var[0] if it's an array.
 */
static ir_dereference *
deref_output(ir_variable *var)
{
   void *mem_ctx = ralloc_parent(var);

   ir_dereference *val = new(mem_ctx) ir_dereference_variable(var);
   if (val->type->is_array()) {
      ir_constant *index = new(mem_ctx) ir_constant(0);
      val = new(mem_ctx) ir_dereference_array(val, index);
   }

   return val;
}

static ir_function_signature *
get_main(gl_linked_shader *sh)
{
   ir_function_signature *sig = NULL;
   /* We can't use _mesa_get_main_function_signature() because we don't
    * have a symbol table at this point.  Just go find main() by hand.
    */
   foreach_in_list(ir_instruction, ir, sh->ir) {
      ir_function *f = ir->as_function();
      if (f && strcmp(f->name, "main") == 0) {
         exec_list void_parameters;
         sig = f->matching_signature(NULL, &void_parameters, false);
         break;
      }
   }
   assert(sig != NULL); /* main() must exist */
   return sig;
}

bool
lower_blend_equation_advanced(struct gl_linked_shader *sh, bool coherent)
{
   if (sh->Program->sh.fs.BlendSupport == 0)
      return false;

   /* Lower early returns in main() so there's a single exit point
    * where we can insert our lowering code.
    */
   do_lower_jumps(sh->ir, false, false, true, false, false);

   void *mem_ctx = ralloc_parent(sh->ir);

   ir_variable *fb = new(mem_ctx) ir_variable(glsl_type::vec4_type,
                                              "__blend_fb_fetch",
                                              ir_var_shader_out);
   fb->data.location = FRAG_RESULT_DATA0;
   fb->data.read_only = 1;
   fb->data.fb_fetch_output = 1;
   fb->data.memory_coherent = coherent;
   fb->data.how_declared = ir_var_hidden;

   ir_variable *mode = new(mem_ctx) ir_variable(glsl_type::uint_type,
                                                "gl_AdvancedBlendModeMESA",
                                                ir_var_uniform);
   mode->data.how_declared = ir_var_hidden;
   mode->allocate_state_slots(1);
   ir_state_slot *slot0 = &mode->get_state_slots()[0];
   slot0->swizzle = SWIZZLE_XXXX;
   slot0->tokens[0] = STATE_INTERNAL;
   slot0->tokens[1] = STATE_ADVANCED_BLENDING_MODE;
   for (int i = 2; i < STATE_LENGTH; i++)
      slot0->tokens[i] = 0;

   sh->ir->push_head(fb);
   sh->ir->push_head(mode);

   /* Gather any output variables referring to render target 0.
    *
    * ARB_enhanced_layouts irritatingly allows the shader to specify
    * multiple output variables for the same render target, each of
    * which writes a subset of the components, starting at location_frac.
    * The variables can't overlap, thankfully.
    */
   ir_variable *outputs[4] = { NULL, NULL, NULL, NULL };
   foreach_in_list(ir_instruction, ir, sh->ir) {
      ir_variable *var = ir->as_variable();
      if (!var || var->data.mode != ir_var_shader_out)
         continue;

      if (var->data.location == FRAG_RESULT_DATA0 ||
          var->data.location == FRAG_RESULT_COLOR) {
         const int components = var->type->without_array()->vector_elements;

         for (int i = 0; i < components; i++) {
            outputs[var->data.location_frac + i] = var;
         }
      }
   }

   /* Combine values written to outputs into a single RGBA blend source.
    * We assign <0, 0, 0, 1> to any components with no corresponding output.
    */
   ir_rvalue *blend_source;
   if (outputs[0] && outputs[0]->type->without_array()->vector_elements == 4) {
      blend_source = deref_output(outputs[0]);
   } else {
      ir_rvalue *blend_comps[4];
      for (int i = 0; i < 4; i++) {
         ir_variable *var = outputs[i];
         if (var) {
            blend_comps[i] = swizzle(deref_output(outputs[i]),
                                     i - outputs[i]->data.location_frac, 1);
         } else {
            blend_comps[i] = new(mem_ctx) ir_constant(i < 3 ? 0.0f : 1.0f);
         }
      }

      blend_source =
         new(mem_ctx) ir_expression(ir_quadop_vector, glsl_type::vec4_type,
                                    blend_comps[0], blend_comps[1],
                                    blend_comps[2], blend_comps[3]);
   }

   ir_function_signature *main = get_main(sh);
   ir_factory f(&main->body, mem_ctx);

   ir_variable *result_dest =
      calc_blend_result(f, mode, fb, blend_source,
                        sh->Program->sh.fs.BlendSupport);

   /* Copy the result back to the original values.  It would be simpler
    * to demote the program's output variables, and create a new vec4
    * output for our result, but this pass runs before we create the
    * ARB_program_interface_query resource list.  So we have to leave
    * the original outputs in place and use them.
    */
   for (int i = 0; i < 4; i++) {
      if (!outputs[i])
         continue;

      f.emit(assign(deref_output(outputs[i]), swizzle(result_dest, i, 1),
                    1 << i));
   }

   validate_ir_tree(sh->ir);
   return true;
}
