/**************************************************************************
 *
 * Copyright 2008-2010 Vmware, Inc.
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


#include "os_misc.h"

#include <stdarg.h>


#if DETECT_OS_WINDOWS

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN      // Exclude rarely-used stuff from Windows headers
#endif
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>

#else

#include <stdio.h>
#include <stdlib.h>

#endif


#if DETECT_OS_ANDROID
#  define LOG_TAG "MESA"
#  include <unistd.h>
#  include <log/log.h>
#elif DETECT_OS_LINUX || DETECT_OS_CYGWIN || DETECT_OS_SOLARIS || DETECT_OS_HURD
#  include <unistd.h>
#elif DETECT_OS_APPLE || DETECT_OS_BSD
#  include <sys/sysctl.h>
#elif DETECT_OS_HAIKU
#  include <kernel/OS.h>
#elif DETECT_OS_WINDOWS
#  include <windows.h>
#else
#error unexpected platform in os_sysinfo.c
#endif


void
os_log_message(const char *message)
{
   /* If the GALLIUM_LOG_FILE environment variable is set to a valid filename,
    * write all messages to that file.
    */
   static FILE *fout = NULL;

   if (!fout) {
#ifdef DEBUG
      /* one-time init */
      const char *filename = os_get_option("GALLIUM_LOG_FILE");
      if (filename) {
         const char *mode = "w";
         if (filename[0] == '+') {
            /* If the filename is prefixed with '+' then open the file for
             * appending instead of normal writing.
             */
            mode = "a";
            filename++; /* skip the '+' */
         }
         fout = fopen(filename, mode);
      }
#endif
      if (!fout)
         fout = stderr;
   }

#if DETECT_OS_WINDOWS
   OutputDebugStringA(message);
   if(GetConsoleWindow() && !IsDebuggerPresent()) {
      fflush(stdout);
      fputs(message, fout);
      fflush(fout);
   }
   else if (fout != stderr) {
      fputs(message, fout);
      fflush(fout);
   }
#else /* !DETECT_OS_WINDOWS */
   fflush(stdout);
   fputs(message, fout);
   fflush(fout);
#  if DETECT_OS_ANDROID
   LOG_PRI(ANDROID_LOG_ERROR, LOG_TAG, "%s", message);
#  endif
#endif
}


#if !defined(EMBEDDED_DEVICE)
const char *
os_get_option(const char *name)
{
   return getenv(name);
}
#endif /* !EMBEDDED_DEVICE */


/**
 * Return the size of the total physical memory.
 * \param size returns the size of the total physical memory
 * \return true for success, or false on failure
 */
bool
os_get_total_physical_memory(uint64_t *size)
{
#if DETECT_OS_LINUX || DETECT_OS_CYGWIN || DETECT_OS_SOLARIS || DETECT_OS_HURD
   const long phys_pages = sysconf(_SC_PHYS_PAGES);
   const long page_size = sysconf(_SC_PAGE_SIZE);

   if (phys_pages <= 0 || page_size <= 0)
      return false;

   *size = (uint64_t)phys_pages * (uint64_t)page_size;
   return true;
#elif DETECT_OS_APPLE || DETECT_OS_BSD
   size_t len = sizeof(*size);
   int mib[2];

   mib[0] = CTL_HW;
#if DETECT_OS_APPLE
   mib[1] = HW_MEMSIZE;
#elif DETECT_OS_NETBSD || DETECT_OS_OPENBSD
   mib[1] = HW_PHYSMEM64;
#elif DETECT_OS_FREEBSD
   mib[1] = HW_REALMEM;
#elif DETECT_OS_DRAGONFLY
   mib[1] = HW_PHYSMEM;
#else
#error Unsupported *BSD
#endif

   return (sysctl(mib, 2, size, &len, NULL, 0) == 0);
#elif DETECT_OS_HAIKU
   system_info info;
   status_t ret;

   ret = get_system_info(&info);
   if (ret != B_OK || info.max_pages <= 0)
      return false;

   *size = (uint64_t)info.max_pages * (uint64_t)B_PAGE_SIZE;
   return true;
#elif DETECT_OS_WINDOWS
   MEMORYSTATUSEX status;
   BOOL ret;

   status.dwLength = sizeof(status);
   ret = GlobalMemoryStatusEx(&status);
   *size = status.ullTotalPhys;
   return (ret == TRUE);
#else
#error unexpected platform in os_sysinfo.c
   return false;
#endif
}
