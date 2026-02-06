/**************************************************************************
 * 
 * Copyright 2008 VMware, Inc.
 * All Rights Reserved.
 * 
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the
 * "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sub license, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 * 
 * The above copyright notice and this permission notice (including the
 * next paragraph) shall be included in all copies or substantial portions
 * of the Software.
 * 
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NON-INFRINGEMENT.
 * IN NO EVENT SHALL VMWARE AND/OR ITS SUPPLIERS BE LIABLE FOR
 * ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 * TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
 * SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 * 
 **************************************************************************/



#include "pipe/p_config.h"
#include "util/u_math.h"
#include "util/u_cpu_detect.h"

#if defined(PIPE_ARCH_SSE)
#include <xmmintrin.h>
/* This is defined in pmmintrin.h, but it can only be included when -msse3 is
 * used, so just define it here to avoid further. */
#ifndef _MM_DENORMALS_ZERO_MASK
#define _MM_DENORMALS_ZERO_MASK	0x0040
#endif
#endif


/** 2^x, for x in [-1.0, 1.0) */
float pow2_table[POW2_TABLE_SIZE];


static void
init_pow2_table(void)
{
   int i;
   for (i = 0; i < POW2_TABLE_SIZE; i++)
      pow2_table[i] = exp2f((i - POW2_TABLE_OFFSET) / POW2_TABLE_SCALE);
}


/** log2(x), for x in [1.0, 2.0) */
float log2_table[LOG2_TABLE_SIZE];


static void 
init_log2_table(void)
{
   unsigned i;
   for (i = 0; i < LOG2_TABLE_SIZE; i++)
      log2_table[i] = (float) log2(1.0 + i * (1.0 / LOG2_TABLE_SCALE));
}


/**
 * One time init for math utilities.
 */
void
util_init_math(void)
{
   static bool initialized = false;
   if (!initialized) {
      init_pow2_table();
      init_log2_table();
      initialized = true;
   }
}

/**
 * Fetches the contents of the fpstate (mxcsr on x86) register.
 *
 * On platforms without support for it just returns 0.
 */
unsigned
util_fpstate_get(void)
{
   unsigned mxcsr = 0;

#if defined(PIPE_ARCH_SSE)
   if (util_cpu_caps.has_sse) {
      mxcsr = _mm_getcsr();
   }
#endif

   return mxcsr;
}

/**
 * Make sure that the fp treats the denormalized floating
 * point numbers as zero.
 *
 * This is the behavior required by D3D10. OpenGL doesn't care.
 */
unsigned
util_fpstate_set_denorms_to_zero(unsigned current_mxcsr)
{
#if defined(PIPE_ARCH_SSE)
   if (util_cpu_caps.has_sse) {
      /* Enable flush to zero mode */
      current_mxcsr |= _MM_FLUSH_ZERO_MASK;
      if (util_cpu_caps.has_daz) {
         /* Enable denormals are zero mode */
         current_mxcsr |= _MM_DENORMALS_ZERO_MASK;
      }
      util_fpstate_set(current_mxcsr);
   }
#endif
   return current_mxcsr;
}

/**
 * Set the state of the fpstate (mxcsr on x86) register.
 *
 * On platforms without support for it's a noop.
 */
void
util_fpstate_set(unsigned mxcsr)
{
#if defined(PIPE_ARCH_SSE)
   if (util_cpu_caps.has_sse) {
      _mm_setcsr(mxcsr);
   }
#endif
}
