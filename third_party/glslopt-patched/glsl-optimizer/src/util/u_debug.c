/**************************************************************************
 *
 * Copyright 2008 VMware, Inc.
 * Copyright (c) 2008 VMware, Inc.
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

#include "util/u_debug.h"
#include "pipe/p_format.h"
#include "pipe/p_state.h"
#include "util/u_string.h"
#include "util/u_math.h"
#include <inttypes.h>

#include <stdio.h>
#include <limits.h> /* CHAR_BIT */
#include <ctype.h> /* isalnum */

#ifdef _WIN32
#include <windows.h>
#include <stdlib.h>
#endif


void
_debug_vprintf(const char *format, va_list ap)
{
   static char buf[4096] = {'\0'};
#if DETECT_OS_WINDOWS || defined(EMBEDDED_DEVICE)
   /* We buffer until we find a newline. */
   size_t len = strlen(buf);
   int ret = vsnprintf(buf + len, sizeof(buf) - len, format, ap);
   if (ret > (int)(sizeof(buf) - len - 1) || strchr(buf + len, '\n')) {
      os_log_message(buf);
      buf[0] = '\0';
   }
#else
   vsnprintf(buf, sizeof(buf), format, ap);
   os_log_message(buf);
#endif
}


void
_pipe_debug_message(struct pipe_debug_callback *cb,
                    unsigned *id,
                    enum pipe_debug_type type,
                    const char *fmt, ...)
{
   va_list args;
   va_start(args, fmt);
   if (cb && cb->debug_message)
      cb->debug_message(cb->data, id, type, fmt, args);
   va_end(args);
}


void
debug_disable_error_message_boxes(void)
{
#ifdef _WIN32
   /* When Windows' error message boxes are disabled for this process (as is
    * typically the case when running tests in an automated fashion) we disable
    * CRT message boxes too.
    */
   UINT uMode = SetErrorMode(0);
   SetErrorMode(uMode);
   if (uMode & SEM_FAILCRITICALERRORS) {
      /* Disable assertion failure message box.
       * http://msdn.microsoft.com/en-us/library/sas1dkb2.aspx
       */
      _set_error_mode(_OUT_TO_STDERR);
#ifdef _MSC_VER
      /* Disable abort message box.
       * http://msdn.microsoft.com/en-us/library/e631wekh.aspx
       */
      _set_abort_behavior(0, _WRITE_ABORT_MSG | _CALL_REPORTFAULT);
#endif
   }
#endif /* _WIN32 */
}


#ifdef DEBUG
void
debug_print_blob(const char *name, const void *blob, unsigned size)
{
   const unsigned *ublob = (const unsigned *)blob;
   unsigned i;

   debug_printf("%s (%d dwords%s)\n", name, size/4,
                size%4 ? "... plus a few bytes" : "");

   for (i = 0; i < size/4; i++) {
      debug_printf("%d:\t%08x\n", i, ublob[i]);
   }
}
#endif


static bool
debug_get_option_should_print(void)
{
   static bool first = true;
   static bool value = false;

   if (!first)
      return value;

   /* Oh hey this will call into this function,
    * but its cool since we set first to false
    */
   first = false;
   value = debug_get_bool_option("GALLIUM_PRINT_OPTIONS", false);
   /* XXX should we print this option? Currently it wont */
   return value;
}


const char *
debug_get_option(const char *name, const char *dfault)
{
   const char *result;

   result = os_get_option(name);
   if (!result)
      result = dfault;

   if (debug_get_option_should_print())
      debug_printf("%s: %s = %s\n", __FUNCTION__, name,
                   result ? result : "(null)");

   return result;
}


bool
debug_get_bool_option(const char *name, bool dfault)
{
   const char *str = os_get_option(name);
   bool result;

   if (str == NULL)
      result = dfault;
   else if (!strcmp(str, "n"))
      result = false;
   else if (!strcmp(str, "no"))
      result = false;
   else if (!strcmp(str, "0"))
      result = false;
   else if (!strcmp(str, "f"))
      result = false;
   else if (!strcmp(str, "F"))
      result = false;
   else if (!strcmp(str, "false"))
      result = false;
   else if (!strcmp(str, "FALSE"))
      result = false;
   else
      result = true;

   if (debug_get_option_should_print())
      debug_printf("%s: %s = %s\n", __FUNCTION__, name,
                   result ? "TRUE" : "FALSE");

   return result;
}


long
debug_get_num_option(const char *name, long dfault)
{
   long result;
   const char *str;

   str = os_get_option(name);
   if (!str) {
      result = dfault;
   } else {
      char *endptr;

      result = strtol(str, &endptr, 0);
      if (str == endptr) {
         /* Restore the default value when no digits were found. */
         result = dfault;
      }
   }

   if (debug_get_option_should_print())
      debug_printf("%s: %s = %li\n", __FUNCTION__, name, result);

   return result;
}


static bool
str_has_option(const char *str, const char *name)
{
   /* Empty string. */
   if (!*str) {
      return false;
   }

   /* OPTION=all */
   if (!strcmp(str, "all")) {
      return true;
   }

   /* Find 'name' in 'str' surrounded by non-alphanumeric characters. */
   {
      const char *start = str;
      unsigned name_len = strlen(name);

      /* 'start' is the beginning of the currently-parsed word,
       * we increment 'str' each iteration.
       * if we find either the end of string or a non-alphanumeric character,
       * we compare 'start' up to 'str-1' with 'name'. */

      while (1) {
         if (!*str || !(isalnum(*str) || *str == '_')) {
            if (str-start == name_len &&
                !memcmp(start, name, name_len)) {
               return true;
            }

            if (!*str) {
               return false;
            }

            start = str+1;
         }

         str++;
      }
   }

   return false;
}


uint64_t
debug_get_flags_option(const char *name,
                       const struct debug_named_value *flags,
                       uint64_t dfault)
{
   uint64_t result;
   const char *str;
   const struct debug_named_value *orig = flags;
   unsigned namealign = 0;

   str = os_get_option(name);
   if (!str)
      result = dfault;
   else if (!strcmp(str, "help")) {
      result = dfault;
      _debug_printf("%s: help for %s:\n", __FUNCTION__, name);
      for (; flags->name; ++flags)
         namealign = MAX2(namealign, strlen(flags->name));
      for (flags = orig; flags->name; ++flags)
         _debug_printf("| %*s [0x%0*"PRIx64"]%s%s\n", namealign, flags->name,
                      (int)sizeof(uint64_t)*CHAR_BIT/4, flags->value,
                      flags->desc ? " " : "", flags->desc ? flags->desc : "");
   }
   else {
      result = 0;
      while (flags->name) {
	 if (str_has_option(str, flags->name))
	    result |= flags->value;
	 ++flags;
      }
   }

   if (debug_get_option_should_print()) {
      if (str) {
         debug_printf("%s: %s = 0x%"PRIx64" (%s)\n",
                      __FUNCTION__, name, result, str);
      } else {
         debug_printf("%s: %s = 0x%"PRIx64"\n", __FUNCTION__, name, result);
      }
   }

   return result;
}


void
_debug_assert_fail(const char *expr, const char *file, unsigned line,
                   const char *function)
{
   _debug_printf("%s:%u:%s: Assertion `%s' failed.\n",
                 file, line, function, expr);
   os_abort();
}


const char *
debug_dump_enum(const struct debug_named_value *names,
                unsigned long value)
{
   static char rest[64];

   while (names->name) {
      if (names->value == value)
	 return names->name;
      ++names;
   }

   snprintf(rest, sizeof(rest), "0x%08lx", value);
   return rest;
}


const char *
debug_dump_enum_noprefix(const struct debug_named_value *names,
                         const char *prefix,
                         unsigned long value)
{
   static char rest[64];

   while (names->name) {
      if (names->value == value) {
         const char *name = names->name;
         while (*name == *prefix) {
            name++;
            prefix++;
         }
         return name;
      }
      ++names;
   }

   snprintf(rest, sizeof(rest), "0x%08lx", value);
   return rest;
}


const char *
debug_dump_flags(const struct debug_named_value *names, unsigned long value)
{
   static char output[4096];
   static char rest[256];
   int first = 1;

   output[0] = '\0';

   while (names->name) {
      if ((names->value & value) == names->value) {
	 if (!first)
	    strncat(output, "|", sizeof(output) - strlen(output) - 1);
	 else
	    first = 0;
	 strncat(output, names->name, sizeof(output) - strlen(output) - 1);
	 output[sizeof(output) - 1] = '\0';
	 value &= ~names->value;
      }
      ++names;
   }

   if (value) {
      if (!first)
	 strncat(output, "|", sizeof(output) - strlen(output) - 1);
      else
	 first = 0;

      snprintf(rest, sizeof(rest), "0x%08lx", value);
      strncat(output, rest, sizeof(output) - strlen(output) - 1);
      output[sizeof(output) - 1] = '\0';
   }

   if (first)
      return "0";

   return output;
}



#ifdef DEBUG
int fl_indent = 0;
const char* fl_function[1024];

int
debug_funclog_enter(const char* f, UNUSED const int line,
                    UNUSED const char* file)
{
   int i;

   for (i = 0; i < fl_indent; i++)
      debug_printf("  ");
   debug_printf("%s\n", f);

   assert(fl_indent < 1023);
   fl_function[fl_indent++] = f;

   return 0;
}

void
debug_funclog_exit(const char* f, UNUSED const int line,
                   UNUSED const char* file)
{
   --fl_indent;
   assert(fl_indent >= 0);
   assert(fl_function[fl_indent] == f);
}

void
debug_funclog_enter_exit(const char* f, UNUSED const int line,
                         UNUSED const char* file)
{
   int i;
   for (i = 0; i < fl_indent; i++)
      debug_printf("  ");
   debug_printf("%s\n", f);
}
#endif
