/***********************************************************************
 * $Id$
 * Copyright 2009 Aplix Corporation. All rights reserved.
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *     http://www.apache.org/licenses/LICENSE-2.0
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 ***********************************************************************/
#ifndef misc_h
#define misc_h
#include <stdarg.h>
#include <stdlib.h>

void *memalloc(size_t size);
void *memrealloc(void *ptr, size_t size);
void memfree(void *ptr);

char *vmemprintf(const char *format, va_list ap);
char *memprintf(const char *format, ...);

void vlocerrorexit(const char *filename, unsigned int linenum, const char *format, va_list ap);
void locerrorexit(const char *filename, unsigned int linenum, const char *format, ...);
void errorexit(const char *format, ...);

#endif /* ndef misc_h */

