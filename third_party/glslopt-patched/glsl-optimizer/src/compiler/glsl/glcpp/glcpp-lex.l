%{
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
#include <ctype.h>

#include "glcpp.h"
#include "glcpp-parse.h"

/* Flex annoyingly generates some functions without making them
 * static. Let's declare them here. */
int glcpp_get_column  (yyscan_t yyscanner);
void glcpp_set_column (int  column_no , yyscan_t yyscanner);

#ifdef _MSC_VER
#define YY_NO_UNISTD_H
#endif

#define YY_NO_INPUT

#define YY_USER_ACTION							\
	do {								\
		if (parser->has_new_line_number)			\
			yylineno = parser->new_line_number;		\
		if (parser->has_new_source_number)			\
			yylloc->source = parser->new_source_number;	\
		yylloc->first_column = yycolumn + 1;			\
		yylloc->first_line = yylloc->last_line = yylineno;	\
		yycolumn += yyleng;					\
		yylloc->last_column = yycolumn + 1;			\
		parser->has_new_line_number = 0;			\
		parser->has_new_source_number = 0;			\
	} while(0);

#define YY_USER_INIT			\
	do {				\
		yylineno = 1;		\
		yycolumn = 0;		\
		yylloc->source = 0;	\
	} while(0)

/* It's ugly to have macros that have return statements inside of
 * them, but flex-based lexer generation is all built around the
 * return statement.
 *
 * To mitigate the ugliness, we defer as much of the logic as possible
 * to an actual function, not a macro (see
 * glcpplex_update_state_per_token) and we make the word RETURN
 * prominent in all of the macros which may return.
 *
 * The most-commonly-used macro is RETURN_TOKEN which will perform all
 * necessary state updates based on the provided token,, then
 * conditionally return the token. It will not return a token if the
 * parser is currently skipping tokens, (such as within #if
 * 0...#else).
 *
 * The RETURN_TOKEN_NEVER_SKIP macro is a lower-level variant that
 * makes the token returning unconditional. This is needed for things
 * like #if and the tokens of its condition, (since these must be
 * evaluated by the parser even when otherwise skipping).
 *
 * Finally, RETURN_STRING_TOKEN is a simple convenience wrapper on top
 * of RETURN_TOKEN that performs a string copy of yytext before the
 * return.
 */
#define RETURN_TOKEN_NEVER_SKIP(token)					\
	do {								\
		if (glcpp_lex_update_state_per_token (parser, token))	\
			return token;					\
	} while (0)

#define RETURN_TOKEN(token)						\
	do {								\
		if (! parser->skipping) {				\
			RETURN_TOKEN_NEVER_SKIP(token);			\
		}							\
	} while(0)

#define RETURN_STRING_TOKEN(token)					\
	do {								\
		if (! parser->skipping) {				\
			/* We're not doing linear_strdup here, to avoid \
			 * an implicit call on strlen() for the length  \
			 * of the string, as this is already found by   \
			 * flex and stored in yyleng */                 \
			void *mem_ctx = yyextra->linalloc;		\
			yylval->str = linear_alloc_child(mem_ctx,	\
							 yyleng + 1);	\
			memcpy(yylval->str, yytext, yyleng + 1);        \
			RETURN_TOKEN_NEVER_SKIP (token);		\
		}							\
	} while(0)


/* Update all state necessary for each token being returned.
 *
 * Here we'll be tracking newlines and spaces so that the lexer can
 * alter its behavior as necessary, (for example, '#' has special
 * significance if it is the first non-whitespace, non-comment token
 * in a line, but does not otherwise).
 *
 * NOTE: If this function returns FALSE, then no token should be
 * returned at all. This is used to suprress duplicate SPACE tokens.
 */
static int
glcpp_lex_update_state_per_token (glcpp_parser_t *parser, int token)
{
	if (token != NEWLINE && token != SPACE && token != HASH_TOKEN &&
	    !parser->lexing_version_directive) {
		glcpp_parser_resolve_implicit_version(parser);
	}

	/* After the first non-space token in a line, we won't
	 * allow any '#' to introduce a directive. */
	if (token == NEWLINE) {
		parser->first_non_space_token_this_line = 1;
	} else if (token != SPACE) {
		parser->first_non_space_token_this_line = 0;
	}

	/* Track newlines just to know whether a newline needs
	 * to be inserted if end-of-file comes early. */
	if (token == NEWLINE) {
		parser->last_token_was_newline = 1;
	} else {
		parser->last_token_was_newline = 0;
	}

	/* Track spaces to avoid emitting multiple SPACE
	 * tokens in a row. */
	if (token == SPACE) {
		if (! parser->last_token_was_space) {
			parser->last_token_was_space = 1;
			return 1;
		} else {
			parser->last_token_was_space = 1;
			return 0;
		}
	} else {
		parser->last_token_was_space = 0;
		return 1;
	}
}


%}

%option bison-bridge bison-locations reentrant noyywrap
%option extra-type="glcpp_parser_t *"
%option prefix="glcpp_"
%option stack
%option never-interactive
%option warn nodefault

	/* Note: When adding any start conditions to this list, you must also
	 * update the "Internal compiler error" catch-all rule near the end of
	 * this file. */

%x COMMENT DEFINE DONE HASH NEWLINE_CATCHUP UNREACHABLE

SPACE		[[:space:]]
NONSPACE	[^[:space:]]
HSPACE		[ \t\v\f]
HASH		#
NEWLINE		(\r\n|\n\r|\r|\n)
IDENTIFIER	[_a-zA-Z][_a-zA-Z0-9]*
PP_NUMBER	[.]?[0-9]([._a-zA-Z0-9]|[eEpP][-+])*
PUNCTUATION	[][(){}.&*~!/%<>^|;,=+-]

/* The OTHER class is simply a catch-all for things that the CPP
parser just doesn't care about. Since flex regular expressions that
match longer strings take priority over those matching shorter
strings, we have to be careful to avoid OTHER matching and hiding
something that CPP does care about. So we simply exclude all
characters that appear in any other expressions. */

OTHER		[^][_#[:space:]#a-zA-Z0-9(){}.&*~!/%<>^|;,=+-]

DIGITS			[0-9][0-9]*
DECIMAL_INTEGER		[1-9][0-9]*[uU]?
OCTAL_INTEGER		0[0-7]*[uU]?
HEXADECIMAL_INTEGER	0[xX][0-9a-fA-F]+[uU]?
PATH			["][]^./ _A-Za-z0-9+*%[(){}|&~=!:;,?-]*["]

%%

	glcpp_parser_t *parser = yyextra;

	/* When we lex a multi-line comment, we replace it (as
	 * specified) with a single space. But if the comment spanned
	 * multiple lines, then subsequent parsing stages will not
	 * count correct line numbers. To avoid this problem we keep
	 * track of all newlines that were commented out by a
	 * multi-line comment, and we emit a NEWLINE token for each at
	 * the next legal opportunity, (which is when the lexer would
	 * be emitting a NEWLINE token anyway).
	 */
	if (YY_START == NEWLINE_CATCHUP) {
		if (parser->commented_newlines)
			parser->commented_newlines--;
		if (parser->commented_newlines == 0)
			BEGIN INITIAL;
		RETURN_TOKEN_NEVER_SKIP (NEWLINE);
	}

	/* Set up the parser->skipping bit here before doing any lexing.
	 *
	 * This bit controls whether tokens are skipped, (as implemented by
         * RETURN_TOKEN), such as between "#if 0" and "#endif".
	 *
	 * The parser maintains a skip_stack indicating whether we should be
         * skipping, (and nested levels of #if/#ifdef/#ifndef/#endif) will
         * push and pop items from the stack.
	 *
	 * Here are the rules for determining whether we are skipping:
	 *
	 *	1. If the skip stack is NULL, we are outside of all #if blocks
	 *         and we are not skipping.
	 *
	 *	2. If the skip stack is non-NULL, the type of the top node in
	 *	   the stack determines whether to skip. A type of
	 *	   SKIP_NO_SKIP is used for blocks wheere we are emitting
	 *	   tokens, (such as between #if 1 and #endif, or after the
	 *	   #else of an #if 0, etc.).
	 *
	 *	3. The lexing_directive bit overrides the skip stack. This bit
	 *	   is set when we are actively lexing the expression for a
	 *	   pre-processor condition, (such as #if, #elif, or #else). In
	 *	   this case, even if otherwise skipping, we need to emit the
	 *	   tokens for this condition so that the parser can evaluate
	 *	   the expression. (For, #else, there's no expression, but we
	 *	   emit tokens so the parser can generate a nice error message
	 *	   if there are any tokens here).
	 */
	if (parser->skip_stack &&
	    parser->skip_stack->type != SKIP_NO_SKIP &&
	    ! parser->lexing_directive)
	{
		parser->skipping = 1;
	} else {
		parser->skipping = 0;
	}

	/* Single-line comments */
<INITIAL,DEFINE,HASH>"//"[^\r\n]* {
}

	/* Multi-line comments */
<INITIAL,DEFINE,HASH>"/*"   { yy_push_state(COMMENT, yyscanner); }
<COMMENT>[^*\r\n]*
<COMMENT>[^*\r\n]*{NEWLINE} { yylineno++; yycolumn = 0; parser->commented_newlines++; }
<COMMENT>"*"+[^*/\r\n]*
<COMMENT>"*"+[^*/\r\n]*{NEWLINE} { yylineno++; yycolumn = 0; parser->commented_newlines++; }
<COMMENT>"*"+"/"        {
	yy_pop_state(yyscanner);
	/* In the <HASH> start condition, we don't want any SPACE token. */
	if (yyextra->space_tokens && YY_START != HASH)
		RETURN_TOKEN (SPACE);
}

{HASH} {

	/* If the '#' is the first non-whitespace, non-comment token on this
	 * line, then it introduces a directive, switch to the <HASH> start
	 * condition.
	 *
	 * Otherwise, this is just punctuation, so return the HASH_TOKEN
         * token. */
	if (parser->first_non_space_token_this_line) {
		BEGIN HASH;
		yyextra->in_define = false;
	}

	RETURN_TOKEN_NEVER_SKIP (HASH_TOKEN);
}

<HASH>version{HSPACE}+ {
	BEGIN INITIAL;
	yyextra->space_tokens = 0;
	yyextra->lexing_version_directive = 1;
	RETURN_STRING_TOKEN (VERSION_TOKEN);
}

	/* Swallow empty #pragma directives, (to avoid confusing the
	 * downstream compiler).
	 *
	 * Note: We use a simple regular expression for the lookahead
	 * here. Specifically, we cannot use the complete {NEWLINE} expression
	 * since it uses alternation and we've found that there's a flex bug
	 * where using alternation in the lookahead portion of a pattern
	 * triggers a buffer overrun. */
<HASH>pragma{HSPACE}*/[\r\n] {
	BEGIN INITIAL;
}

	/* glcpp doesn't handle #extension, #version, or #pragma directives.
	 * Simply pass them through to the main compiler's lexer/parser. */
<HASH>(extension|pragma)[^\r\n]* {
	BEGIN INITIAL;
	RETURN_STRING_TOKEN (PRAGMA);
}

<HASH>include{HSPACE}+["<][]^./ _A-Za-z0-9+*%[(){}|&~=!:;,?-]+[">] {
	BEGIN INITIAL;
	RETURN_STRING_TOKEN (INCLUDE);
}

<HASH>line{HSPACE}+ {
	BEGIN INITIAL;
	RETURN_TOKEN (LINE);
}

<HASH>{NEWLINE} {
	BEGIN INITIAL;
	yyextra->space_tokens = 0;
	yylineno++;
	yycolumn = 0;
	RETURN_TOKEN_NEVER_SKIP (NEWLINE);
}

	/* For the pre-processor directives, we return these tokens
	 * even when we are otherwise skipping. */
<HASH>ifdef {
	if (!yyextra->in_define) {
		BEGIN INITIAL;
		yyextra->lexing_directive = 1;
		yyextra->space_tokens = 0;
		RETURN_TOKEN_NEVER_SKIP (IFDEF);
	}
}

<HASH>ifndef {
	if (!yyextra->in_define) {
		BEGIN INITIAL;
		yyextra->lexing_directive = 1;
		yyextra->space_tokens = 0;
		RETURN_TOKEN_NEVER_SKIP (IFNDEF);
	}
}

<HASH>if/[^_a-zA-Z0-9] {
	if (!yyextra->in_define) {
		BEGIN INITIAL;
		yyextra->lexing_directive = 1;
		yyextra->space_tokens = 0;
		RETURN_TOKEN_NEVER_SKIP (IF);
	}
}

<HASH>elif/[^_a-zA-Z0-9] {
	if (!yyextra->in_define) {
		BEGIN INITIAL;
		yyextra->lexing_directive = 1;
		yyextra->space_tokens = 0;
		RETURN_TOKEN_NEVER_SKIP (ELIF);
	}
}

<HASH>else {
	if (!yyextra->in_define) {
		BEGIN INITIAL;
		yyextra->space_tokens = 0;
		RETURN_TOKEN_NEVER_SKIP (ELSE);
	}
}

<HASH>endif {
	if (!yyextra->in_define) {
		BEGIN INITIAL;
		yyextra->space_tokens = 0;
		RETURN_TOKEN_NEVER_SKIP (ENDIF);
	}
}

<HASH>error[^\r\n]* {
	BEGIN INITIAL;
	RETURN_STRING_TOKEN (ERROR_TOKEN);
}

	/* After we see a "#define" we enter the <DEFINE> start state
	 * for the lexer. Within <DEFINE> we are looking for the first
	 * identifier and specifically checking whether the identifier
	 * is followed by a '(' or not, (to lex either a
	 * FUNC_IDENTIFIER or an OBJ_IDENITIFIER token).
	 *
	 * While in the <DEFINE> state we also need to explicitly
	 * handle a few other things that may appear before the
	 * identifier:
	 * 
	 * 	* Comments, (handled above with the main support for
	 * 	  comments).
	 *
	 *	* Whitespace (simply ignored)
	 *
	 *	* Anything else, (not an identifier, not a comment,
	 *	  and not whitespace). This will generate an error.
	 */
<HASH>define{HSPACE}* {
	yyextra->in_define = true;
	if (!parser->skipping) {
		BEGIN DEFINE;
		yyextra->space_tokens = 0;
		RETURN_TOKEN (DEFINE_TOKEN);
	}
}

<HASH>undef {
	BEGIN INITIAL;
	yyextra->space_tokens = 0;
	RETURN_TOKEN (UNDEF);
}

<HASH>{HSPACE}+ {
	/* Nothing to do here. Importantly, don't leave the <HASH>
	 * start condition, since it's legal to have space between the
	 * '#' and the directive.. */
}

	/* This will catch any non-directive garbage after a HASH */
<HASH>{NONSPACE} {
	if (!parser->skipping) {
		BEGIN INITIAL;
		RETURN_TOKEN (GARBAGE);
	}
}

	/* An identifier immediately followed by '(' */
<DEFINE>{IDENTIFIER}/"(" {
	BEGIN INITIAL;
	RETURN_STRING_TOKEN (FUNC_IDENTIFIER);
}

	/* An identifier not immediately followed by '(' */
<DEFINE>{IDENTIFIER} {
	BEGIN INITIAL;
	RETURN_STRING_TOKEN (OBJ_IDENTIFIER);
}

	/* Whitespace */
<DEFINE>{HSPACE}+ {
	/* Just ignore it. Nothing to do here. */
}

	/* '/' not followed by '*', so not a comment. This is an error. */
<DEFINE>[/][^*]{NONSPACE}* {
	BEGIN INITIAL;
	glcpp_error(yylloc, yyextra, "#define followed by a non-identifier: %s", yytext);
	RETURN_STRING_TOKEN (INTEGER_STRING);
}

	/* A character that can't start an identifier, comment, or
	 * space. This is an error. */
<DEFINE>[^_a-zA-Z/[:space:]]{NONSPACE}* {
	BEGIN INITIAL;
	glcpp_error(yylloc, yyextra, "#define followed by a non-identifier: %s", yytext);
	RETURN_STRING_TOKEN (INTEGER_STRING);
}

{DECIMAL_INTEGER} {
	RETURN_STRING_TOKEN (INTEGER_STRING);
}

{OCTAL_INTEGER} {
	RETURN_STRING_TOKEN (INTEGER_STRING);
}

{HEXADECIMAL_INTEGER} {
	RETURN_STRING_TOKEN (INTEGER_STRING);
}

"<<"  {
	RETURN_TOKEN (LEFT_SHIFT);
}

">>" {
	RETURN_TOKEN (RIGHT_SHIFT);
}

"<=" {
	RETURN_TOKEN (LESS_OR_EQUAL);
}

">=" {
	RETURN_TOKEN (GREATER_OR_EQUAL);
}

"==" {
	RETURN_TOKEN (EQUAL);
}

"!=" {
	RETURN_TOKEN (NOT_EQUAL);
}

"&&" {
	RETURN_TOKEN (AND);
}

"||" {
	RETURN_TOKEN (OR);
}

"++" {
	RETURN_TOKEN (PLUS_PLUS);
}

"--" {
	RETURN_TOKEN (MINUS_MINUS);
}

"##" {
	if (! parser->skipping) {
		if (parser->is_gles)
			glcpp_error(yylloc, yyextra, "Token pasting (##) is illegal in GLES");
		RETURN_TOKEN (PASTE);
	}
}

"defined" {
	RETURN_TOKEN (DEFINED);
}

{IDENTIFIER} {
	RETURN_STRING_TOKEN (IDENTIFIER);
}

{PP_NUMBER} {
	RETURN_STRING_TOKEN (OTHER);
}

{PUNCTUATION} {
	RETURN_TOKEN (yytext[0]);
}

{OTHER}+ {
	RETURN_STRING_TOKEN (OTHER);
}

{HSPACE} {
	if (yyextra->space_tokens) {
		RETURN_TOKEN (SPACE);
	}
}

{PATH} {
	RETURN_STRING_TOKEN (PATH);
}

	/* We preserve all newlines, even between #if 0..#endif, so no
	skipping.. */
<*>{NEWLINE} {
	if (parser->commented_newlines) {
		BEGIN NEWLINE_CATCHUP;
	} else {
		BEGIN INITIAL;
	}
	yyextra->space_tokens = 1;
	yyextra->lexing_directive = 0;
	yyextra->lexing_version_directive = 0;
	yylineno++;
	yycolumn = 0;
	RETURN_TOKEN_NEVER_SKIP (NEWLINE);
}

<INITIAL,COMMENT,DEFINE,HASH><<EOF>> {
	if (YY_START == COMMENT)
		glcpp_error(yylloc, yyextra, "Unterminated comment");
	BEGIN DONE; /* Don't keep matching this rule forever. */
	yyextra->lexing_directive = 0;
	yyextra->lexing_version_directive = 0;
	if (! parser->last_token_was_newline)
		RETURN_TOKEN (NEWLINE);
}

	/* This is a catch-all to avoid the annoying default flex action which
	 * matches any character and prints it. If any input ever matches this
	 * rule, then we have made a mistake above and need to fix one or more
	 * of the preceding patterns to match that input. */

<*>. {
	glcpp_error(yylloc, yyextra, "Internal compiler error: Unexpected character: %s", yytext);

	/* We don't actually use the UNREACHABLE start condition. We
	only have this block here so that we can pretend to call some
	generated functions, (to avoid "defined but not used"
	warnings. */
        if (YY_START == UNREACHABLE) {
		unput('.');
		yy_top_state(yyextra);
	}
}

%%

void
glcpp_lex_set_source_string(glcpp_parser_t *parser, const char *shader)
{
	yy_scan_string(shader, parser->scanner);
}
