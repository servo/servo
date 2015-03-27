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
#include <string.h>
#include "misc.h"
#include "process.h"

static const char nodtdopt[] = "-no-dtd-ref";
const char *progname;

/***********************************************************************
 * options : process command line options
 *
 * Enter:   argv
 *
 * Return:  argv stepped to point to first non-option argument
 */
static const char *const *
options(int argc, const char *const *argv)
{
    /* Set progname for error messages etc. */
    {
        const char *base;
        progname = argv[0];
        base = strrchr(progname, '/');
#ifdef DIRSEP
        {
            const char *base2 = strrchr(base, '\\');
            if (base2 > base)
                base = base2;
        }
#endif /* def DIRSEP */
        if (base)
            progname = base + 1;
    }
    return (argc > 1 && strncmp(argv[1], nodtdopt, sizeof nodtdopt) == 0)
          ? argv + 2 : argv + 1;
}

/***********************************************************************
 * main : main code for bondiidl command
 */
int
main(int argc, char **argv)
{
    const char *const *parg;
    parg = options(argc, (const char *const *)argv);
    if (!*parg)
        errorexit("usage: %s [-no-dtd-ref] <interface>.widl ...", progname);
    processfiles(parg, parg == (const char *const *)argv + 1);
    return 0;
}

