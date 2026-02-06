/* Copyright © 2007 Carl Worth
 * Copyright © 2009 Jeremy Huddleston, Julien Cristau, and Matthieu Herrb
 * Copyright © 2009-2010 Mikhail Gusarov
 * Copyright © 2012 Yaakov Selkowitz and Keith Packard
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

#include "sha1/sha1.h"
#include "mesa-sha1.h"

void
_mesa_sha1_compute(const void *data, size_t size, unsigned char result[20])
{
   struct mesa_sha1 ctx;

   _mesa_sha1_init(&ctx);
   _mesa_sha1_update(&ctx, data, size);
   _mesa_sha1_final(&ctx, result);
}

void
_mesa_sha1_format(char *buf, const unsigned char *sha1)
{
   static const char hex_digits[] = "0123456789abcdef";
   int i;

   for (i = 0; i < 40; i += 2) {
      buf[i] = hex_digits[sha1[i >> 1] >> 4];
      buf[i + 1] = hex_digits[sha1[i >> 1] & 0x0f];
   }
   buf[i] = '\0';
}
