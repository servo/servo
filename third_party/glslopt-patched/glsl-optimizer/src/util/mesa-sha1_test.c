/*
 * Copyright © 2017 Intel Corporation
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
 */

#include <stdio.h>
#include <stdbool.h>
#include <string.h>

#include "macros.h"
#include "mesa-sha1.h"

#define SHA1_LENGTH 40

int main(int argc, char *argv[])
{
   static const struct {
      const char *string;
      const char *sha1;
   } test_data[] = {
      {"Mesa Rocks! 273", "7fb99737373d65a73f049cdabc01e73aa6bc60f3"},
      {"Mesa Rocks! 300", "b2180263e37d3bed6a4be0afe41b1a82ebbcf4c3"},
      {"Mesa Rocks! 583", "7fb9734108a62503e8a149c1051facd7fb112d05"},
   };

   bool failed = false;
   int i;

   for (i = 0; i < ARRAY_SIZE(test_data); i++) {
      unsigned char sha1[20];
      _mesa_sha1_compute(test_data[i].string, strlen(test_data[i].string),
                         sha1);

      char buf[41];
      _mesa_sha1_format(buf, sha1);

      if (memcmp(test_data[i].sha1, buf, SHA1_LENGTH) != 0) {
         printf("For string \"%s\", length %zu:\n"
                "\tExpected: %s\n\t     Got: %s\n",
                test_data[i].string, strlen(test_data[i].string),
                test_data[i].sha1, buf);
         failed = true;
      }
   }

   return failed;
}
