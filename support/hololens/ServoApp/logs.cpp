/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"

void log(const char *format, ...) {
  char buf[4096], *p = buf;
  va_list args;
  int n;

  va_start(args, format);
  n = vsnprintf(p, sizeof buf - 3, format, args);
  va_end(args);

  p += (n < 0) ? sizeof buf - 3 : n;

  while (p > buf && isspace(p[-1])) {
    *--p = '\0';
  }

  *p++ = '\r';
  *p++ = '\n';
  *p = '\0';

  OutputDebugStringA(buf);
}