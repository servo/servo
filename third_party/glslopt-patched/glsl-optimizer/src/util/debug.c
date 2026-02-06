/*
 * Copyright © 2015 Intel Corporation
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

#include <errno.h>
#include <string.h>
#include "debug.h"
#include "u_string.h"

uint64_t
parse_debug_string(const char *debug,
                   const struct debug_control *control)
{
   uint64_t flag = 0;

   if (debug != NULL) {
      for (; control->string != NULL; control++) {
         if (!strcmp(debug, "all")) {
            flag |= control->flag;

         } else {
            const char *s = debug;
            unsigned n;

            for (; n = strcspn(s, ", "), *s; s += MAX2(1, n)) {
               if (strlen(control->string) == n &&
                   !strncmp(control->string, s, n))
                  flag |= control->flag;
            }
         }
      }
   }

   return flag;
}

bool
comma_separated_list_contains(const char *list, const char *s)
{
   assert(list);
   const size_t len = strlen(s);

   for (unsigned n; n = strcspn(list, ","), *list; list += MAX2(1, n)) {
      if (n == len && !strncmp(list, s, n))
         return true;
   }

   return false;
}

/**
 * Reads an environment variable and interprets its value as a boolean.
 *
 * Recognizes 0/false/no and 1/true/yes.  Other values result in the default value.
 */
bool
env_var_as_boolean(const char *var_name, bool default_value)
{
   const char *str = getenv(var_name);
   if (str == NULL)
      return default_value;

   if (strcmp(str, "1") == 0 ||
       strcasecmp(str, "true") == 0 ||
       strcasecmp(str, "y") == 0 ||
       strcasecmp(str, "yes") == 0) {
      return true;
   } else if (strcmp(str, "0") == 0 ||
              strcasecmp(str, "false") == 0 ||
              strcasecmp(str, "n") == 0 ||
              strcasecmp(str, "no") == 0) {
      return false;
   } else {
      return default_value;
   }
}

/**
 * Reads an environment variable and interprets its value as a unsigned.
 */
unsigned
env_var_as_unsigned(const char *var_name, unsigned default_value)
{
   char *str = getenv(var_name);
   if (str) {
      char *end;
      unsigned long result;

      errno = 0;
      result = strtoul(str, &end, 0);
      if (errno == 0 && end != str && *end == '\0')
        return result;
   }
   return default_value;
}
