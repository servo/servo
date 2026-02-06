/*
 * Copyright © 2009,2012 Intel Corporation
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
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
 * IN THE SOFTWARE.
 *
 * Authors:
 *    Eric Anholt <eric@anholt.net>
 *
 */

#ifndef _FNV1A_H
#define _FNV1A_H

enum {
   _mesa_fnv32_1a_offset_bias = 2166136261u,
};

/**
 * Quick FNV-1a hash implementation based on:
 * http://www.isthe.com/chongo/tech/comp/fnv/
 *
 * FNV-1a is not be the best hash out there -- Jenkins's lookup3 is supposed
 * to be quite good, and it probably beats FNV.  But FNV has the advantage
 * that it involves almost no code.  For an improvement on both, see Paul
 * Hsieh's http://www.azillionmonkeys.com/qed/hash.html
 */
static inline uint32_t
_mesa_fnv32_1a_accumulate_block(uint32_t hash, const void *data, size_t size)
{
   const uint8_t *bytes = (const uint8_t *)data;

   while (size-- != 0) {
      hash ^= *bytes;
      hash = hash * 0x01000193;
      bytes++;
   }

   return hash;
}

#define _mesa_fnv32_1a_accumulate(hash, expr) \
   _mesa_fnv32_1a_accumulate_block(hash, &(expr), sizeof(expr))

#endif
