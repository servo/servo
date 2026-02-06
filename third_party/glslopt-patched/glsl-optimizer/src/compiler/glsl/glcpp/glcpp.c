/*
 * Copyright © 2010 Intel Corporation
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

#include <stdio.h>
#include <string.h>
#include <errno.h>
#include <getopt.h>

#include "glcpp.h"
#include "main/mtypes.h"
#include "main/shaderobj.h"
#include "util/strtod.h"

extern int glcpp_parser_debug;

void
_mesa_reference_shader(struct gl_context *ctx, struct gl_shader **ptr,
                       struct gl_shader *sh)
{
   (void) ctx;
   *ptr = sh;
}

/* Read from fp until EOF and return a string of everything read.
 */
static char *
load_text_fp (void *ctx, FILE *fp)
{
#define CHUNK 4096
	char *text = NULL;
	size_t text_size = 0;
	size_t total_read = 0;
	size_t bytes;

	while (1) {
		if (total_read + CHUNK + 1 > text_size) {
			text_size = text_size ? text_size * 2 : CHUNK + 1;
			text = reralloc_size (ctx, text, text_size);
			if (text == NULL) {
				fprintf (stderr, "Out of memory\n");
				return NULL;
			}
		}
		bytes = fread (text + total_read, 1, CHUNK, fp);
		total_read += bytes;

		if (bytes < CHUNK) {
			break;
		}
	}

	text[total_read] = '\0';

	return text;
}

static char *
load_text_file(void *ctx, const char *filename)
{
	char *text;
	FILE *fp;

	if (filename == NULL || strcmp (filename, "-") == 0)
		return load_text_fp (ctx, stdin);

	fp = fopen (filename, "r");
	if (fp == NULL) {
		fprintf (stderr, "Failed to open file %s: %s\n",
			 filename, strerror (errno));
		return NULL;
	}

	text = load_text_fp (ctx, fp);

	fclose(fp);

	return text;
}

/* Initialize only those things that glcpp cares about.
 */
static void
init_fake_gl_context (struct gl_context *gl_ctx)
{
	gl_ctx->API = API_OPENGL_COMPAT;
	gl_ctx->Const.DisableGLSLLineContinuations = false;
}

static void
usage (void)
{
	fprintf (stderr,
		 "Usage: glcpp [OPTIONS] [--] [<filename>]\n"
		 "\n"
		 "Pre-process the given filename (stdin if no filename given).\n"
		 "The following options are supported:\n"
		 "    --disable-line-continuations      Do not interpret lines ending with a\n"
		 "                                      backslash ('\\') as a line continuation.\n");
}

enum {
	DISABLE_LINE_CONTINUATIONS_OPT = CHAR_MAX + 1
};

static const struct option
long_options[] = {
	{"disable-line-continuations", no_argument, 0, DISABLE_LINE_CONTINUATIONS_OPT },
        {"debug",                      no_argument, 0, 'd'},
	{0,                            0,           0, 0 }
};

int
main (int argc, char *argv[])
{
	char *filename = NULL;
	void *ctx = ralloc(NULL, void*);
	char *info_log = ralloc_strdup(ctx, "");
	const char *shader;
	int ret;
	struct gl_context gl_ctx;
	int c;

	init_fake_gl_context (&gl_ctx);

	while ((c = getopt_long(argc, argv, "d", long_options, NULL)) != -1) {
		switch (c) {
		case DISABLE_LINE_CONTINUATIONS_OPT:
			gl_ctx.Const.DisableGLSLLineContinuations = true;
			break;
                case 'd':
			glcpp_parser_debug = 1;
			break;
		default:
			usage ();
			exit (1);
		}
	}

	if (optind + 1 < argc) {
		printf ("Unexpected argument: %s\n", argv[optind+1]);
		usage ();
		exit (1);
	}
	if (optind < argc) {
		filename = argv[optind];
	}

	shader = load_text_file (ctx, filename);
	if (shader == NULL)
	   return 1;

	_mesa_locale_init();

	ret = glcpp_preprocess(ctx, &shader, &info_log, NULL, NULL, &gl_ctx);

	printf("%s", shader);
	fprintf(stderr, "%s", info_log);

	ralloc_free(ctx);

	return ret;
}
