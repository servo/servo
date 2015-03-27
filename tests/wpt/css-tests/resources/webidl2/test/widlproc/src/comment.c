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
#include <assert.h>
#include <string.h>
#include <stdio.h>

#include "comment.h"
#include "entities.h"
#include "lex.h"
#include "misc.h"
#include "node.h"
#include "os.h"
#include "process.h"

/* struct cnode : a node in the comment parse tree */
struct cnode {
    struct cnode *next;
    struct cnode *children;
    struct cnode *parent;
    const struct cnodefuncs *funcs;
    const char *attrtext;
    const char *filename;
    unsigned int linenum;
};

struct cnodefuncs {
    int indesc; /* non-zero if it outputs its own xml element that does
                      not want to be inside <description> */
    int needpara; /* non-zero if text must be in a para node that is a child
                     of this one */
    int (*askend)(struct cnode *cnode, const struct cnodefuncs *type);
    void (*end)(struct cnode *cnode);
    void (*output)(struct cnode *cnode, unsigned int indent);
};

struct paramcnode {
    struct cnode cn;
    int inout;
    char name[1];
};

/* struct comment : a doxygen comment */
struct comment {
    struct comment *next;
    struct node *node;
    unsigned int type;
    const char *filename;
    unsigned int linenum;
    struct cnode root;
    int back; /* Whether the comment refers back rather than forward. */
    char *text;
};


static struct node *lastidentifier;
static struct comment *comments;
static int incode, inhtmlblock;
static struct comment *curcomment;

/***********************************************************************
 * htmleldescs : table of recnogized HTML elements
 */
#define HTMLEL_EMPTY 1
#define HTMLEL_INLINE 2
#define HTMLEL_BLOCK 4
#define HTMLEL_AUTOCLOSE 8
#define HTMLEL_LI 0x10
#define HTMLEL_DLCONTENTS 0x20
#define HTMLEL_TABLECONTENTS 0x40
#define HTMLEL_TRCONTENTS 0x80

#define HTMLEL_FLOW (HTMLEL_BLOCK | HTMLEL_INLINE)

struct htmleldesc {
    unsigned int namelen;
    const char *name;
    unsigned int flags;
    unsigned int content;
};

static const struct htmleldesc htmleldescs[] = {
    { 1, "a", HTMLEL_INLINE, 0 },
    { 1, "b", HTMLEL_INLINE, 0 },
    { 2, "br", HTMLEL_INLINE, HTMLEL_EMPTY },
    { 3, "img", HTMLEL_INLINE, HTMLEL_EMPTY },
    { 2, "dd", HTMLEL_DLCONTENTS, HTMLEL_FLOW },
    { 2, "dl", HTMLEL_BLOCK, HTMLEL_DLCONTENTS },
    { 2, "dt", HTMLEL_DLCONTENTS, HTMLEL_INLINE },
    { 2, "em", HTMLEL_INLINE, 0 },
    { 2, "li", HTMLEL_LI, HTMLEL_FLOW },
    { 2, "ol", HTMLEL_BLOCK, HTMLEL_LI },
    { 1, "p", HTMLEL_BLOCK, HTMLEL_INLINE },
    { 2, "td", HTMLEL_TRCONTENTS | HTMLEL_AUTOCLOSE, HTMLEL_FLOW },
    { 2, "th", HTMLEL_TRCONTENTS | HTMLEL_AUTOCLOSE, HTMLEL_FLOW },
    { 2, "tr", HTMLEL_TABLECONTENTS | HTMLEL_AUTOCLOSE, HTMLEL_TRCONTENTS },
    { 5, "table", HTMLEL_BLOCK, HTMLEL_TABLECONTENTS },
    { 2, "ul", HTMLEL_BLOCK, HTMLEL_LI },
    { 0, 0, 0, 0 }
};
#define HTMLELDESC_B (htmleldescs + 1)
#define HTMLELDESC_BR (htmleldescs + 2)

/***********************************************************************
 * addcomment : add a comment to the list of comments if it has doxygen syntax
 *
 * Enter:   tok struct
 */
void
addcomment(struct tok *tok)
{
    if (tok->len >= 1 && (tok->start[0] == '!'
        || (tok->type == TOK_BLOCKCOMMENT && tok->start[0] == '*')
        || (tok->type == TOK_INLINECOMMENT && tok->start[0] == '/')))
    {
        struct comment *comment;
        comment = memalloc(sizeof(struct comment));
        comment->text = memalloc(tok->len + 1);
        memcpy(comment->text, tok->start, tok->len);
        comment->text[tok->len] = 0;
        comment->type = tok->type;
        comment->filename = tok->filename;
        comment->linenum = tok->linenum;
        comment->node = 0;
        comment->back = 0;
        if (comment->text[1] == '<') {
            comment->back = 1;
            if (!lastidentifier) {
                locerrorexit(comment->filename, comment->linenum,
                    "no identifier to attach doxygen comment to");
            }
            comment->node = lastidentifier;
        }
        comment->next = comments;
        comments = comment;
    }
}

/***********************************************************************
 * setcommentnode : set parse node to attach comments to
 *
 * Enter:   node2 = parse node for identifier
 */
void
setcommentnode(struct node *node2)
{
    struct comment *comment = comments;
    while (comment && !comment->node) {
        comment->node = node2;
        comment = comment->next;
    }
    lastidentifier = node2;
}

/***********************************************************************
 * joininlinecomments : join adjacent inline comments
 *
 * Enter:   comment = list of comment structs
 *
 * Return:  new list of comment structs
 *
 * This function also discards any single inline comment that does not
 * refer back.
 */
static struct comment *
joininlinecomments(struct comment *comments)
{
    struct comment **pcomment;
    pcomment = &comments;
    for (;;) {
        struct comment *comment;
        comment = *pcomment;
        if (!comment)
            break;
        if (comment->type != TOK_INLINECOMMENT) {
            /* Keep block comment as is. */
            pcomment = &comment->next;
        } else if (!comment->back && (!comment->next
                || comment->next->type != TOK_INLINECOMMENT
                || comment->next->filename != comment->filename
                || comment->next->linenum != comment->linenum + 1))
        {
            /* Discard single // comment that does not refer back. */
            *pcomment = comment->next;

        } else {
            /* Find sequence of adjacent // comments (adjacent lines,
             * referring to same node) and join them. We do this in two
             * passes, one to count the length of the comment and one
             * to join. Note that the list is still in reverse order,
             * so we expect the line number to decrease by 1 each time. */
            struct comment *newcomment = 0, *comment2;
            const char *filename = comment->filename;
            unsigned int linenum = comment->linenum;
            for (;;) {
                char *wp = newcomment->text;
                comment2 = comment;
                do {
                    unsigned int len = strlen(comment2->text);
                    if (newcomment)
                        memcpy(wp, comment2->text, len);
                    wp += len;
                    linenum--;
                    comment2 = comment2->next;
                } while (comment2 && comment2->filename == filename
                            && comment2->linenum == linenum
                            && comment2->node == comment->node);
                /* Finished a pass. */
                if (newcomment) {
                    *wp = 0;
                    break;
                }
                newcomment = memalloc(sizeof(struct comment)
                                + wp - newcomment->text);
                newcomment->node = comment->node;
                newcomment->type = comment->type;
                newcomment->filename = filename;
                newcomment->linenum = linenum + 1;
            }
            /* Replace the scanned comment struct with newcomment in the
             * list. */
            newcomment->next = comment2;
            *pcomment = newcomment;
            pcomment = &newcomment->next;
        }
    }
    return comments;
}

/***********************************************************************
 * outputchildren : call output recursively on children of cnode
 *
 * Enter:   cnode
 *          indent = indent (nesting) level of parent
 *          indesc = whether already in <description> or other top-level
 *                   descriptive element
 */
static void
outputchildren(struct cnode *cnode, unsigned int indent, int indesc)
{
    int curindesc = indesc;
    cnode = cnode->children;
    while (cnode) {
        if (curindesc != cnode->funcs->indesc) {
            assert(!indesc);
            printf("%*s<%sdescription>\n", indent + 1, "", curindesc ? "/" : "");
            curindesc = !curindesc;
        }
        (*cnode->funcs->output)(cnode, indent + 2);
        cnode = cnode->next;
    }
    if (curindesc != indesc)
        printf("%*s<%sdescription>\n", indent + 1, "", curindesc ? "/" : "");
}

/***********************************************************************
 * default_askend : ask node if it wants to end at a para start (default
 *                  implementation)
 *
 * Enter:   cnode
 *          type = 0 else cnodefuncs for newly starting para
 *
 * Return:  non-zero if this node wants to end
 */
static int
default_askend(struct cnode *cnode, const struct cnodefuncs *type)
{
    return 1;
}

/***********************************************************************
 * root_askend : ask root node if it wants to end at a para start
 *
 * Enter:   cnode for root
 *          type = 0 else cnodefuncs for newly starting para
 *
 * Return:  non-zero if this node wants to end
 */
static int
root_askend(struct cnode *cnode, const struct cnodefuncs *type)
{
    return 0;
}

/***********************************************************************
 * root_output : output root cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
root_output(struct cnode *cnode, unsigned int indent)
{
    outputchildren(cnode, indent, 0);
}

/***********************************************************************
 * cnode type root
 */
static const struct cnodefuncs root_funcs = {
    0, /* !indesc */
    1, /* needpara */
    &root_askend,
    0, /* end */
    &root_output,
};

/***********************************************************************
 * endcnode : end current cnode
 *
 * Enter:   cnode = current cnode
 *
 * Return:  cnode = new current code (parent of old one)
 */
static struct cnode *
endcnode(struct cnode *cnode)
{
    if (cnode->funcs->end)
        (*cnode->funcs->end)(cnode);
    /* Reverse the children list. */
    {
        struct cnode *child = cnode->children;
        cnode->children = 0;
        while (child) {
            struct cnode *next = child->next;
            child->next = cnode->children;
            cnode->children = child;
            child = next;
        }
    }
    return cnode->parent;
}

/***********************************************************************
 * endspecificcnode : end a specific type of cnode
 *
 * Enter:   cnode = current cnode
 *          type = type of node to end
 *          filename, linenum = filename and line number (for error reporting)
 *
 * Return:  new current cnode
 */
static struct cnode *
endspecificcnode(struct cnode *cnode, const struct cnodefuncs *type,
                 const char *filename, unsigned int linenum)
{
    while (cnode->funcs != type) {
        if (cnode->funcs == &root_funcs)
            locerrorexit(filename, linenum, "unmatched \\endcode");
        cnode = endcnode(cnode);
    }
    return cnode;
}

/***********************************************************************
 * startcnode : start a newly created cnode
 *
 * Enter:   cnode = current cnode
 *          newcnode = new cnode to start
 *
 * Return:  new current cnode (which is the same as newcnode)
 */
static struct cnode *
startcnode(struct cnode *cnode, struct cnode *newcnode)
{
    newcnode->parent = cnode;
    newcnode->next = cnode->children;
    cnode->children = newcnode;
    return newcnode;
}

/***********************************************************************
 * para_output : output para cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
para_output(struct cnode *cnode, unsigned int indent)
{
    printf("%*s<p>\n", indent, "");
    outputchildren(cnode, indent, 1);
    printf("%*s</p>\n", indent, "");
}

/***********************************************************************
 * para_end : end a para cnode
 *
 * Enter:   cnode struct
 */
static void
para_end(struct cnode *cnode)
{
    /* If the para cnode is empty, remove it. */
    if (!cnode->children)
        cnode->parent->children = cnode->next;
}

/***********************************************************************
 * cnode type para
 */
static const struct cnodefuncs para_funcs = {
    1, /* indesc */
    0, /* !needpara */
    &default_askend,
    &para_end, /* end */
    &para_output,
};

/***********************************************************************
 * brief_output : output brief cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
brief_output(struct cnode *cnode, unsigned int indent)
{
    printf("%*s<brief>\n", indent, "");
    outputchildren(cnode, indent, 1);
    printf("%*s</brief>\n", indent, "");
}

/***********************************************************************
 * cnode type brief
 */
static const struct cnodefuncs brief_funcs = {
    0, /* !indesc */
    0, /* !needpara */
    &default_askend,
    0, /* end */
    &brief_output,
};

/***********************************************************************
 * return_output : output return cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
return_output(struct cnode *cnode, unsigned int indent)
{
    printf("%*s<description><p>\n", indent, "");
    outputchildren(cnode, indent, 1);
    printf("%*s</p></description>\n", indent, "");
}

/***********************************************************************
 * cnode type return
 */
static const struct cnodefuncs return_funcs = {
    0, /* !indesc */
    0, /* !needpara */
    &default_askend,
    0, /* end */
    &return_output,
};

/***********************************************************************
 * author_output : output name cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
name_output(struct cnode *cnode, unsigned int indent)
{
    printf("%*s<name>\n", indent, "");
    outputchildren(cnode, indent, 1);
    printf("%*s</name>\n", indent, "");
}

/***********************************************************************
 * cnode type name
 */
static const struct cnodefuncs name_funcs = {
    0, /* !indesc */
    0, /* !needpara */
    &default_askend,
    0, /* end */
    &name_output,
};

/***********************************************************************
 * author_output : output author cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
author_output(struct cnode *cnode, unsigned int indent)
{
    printf("%*s<author>\n", indent, "");
    outputchildren(cnode, indent, 1);
    printf("%*s</author>\n", indent, "");
}

/***********************************************************************
 * cnode type author
 */
static const struct cnodefuncs author_funcs = {
    0, /* !indesc */
    0, /* !needpara */
    &default_askend,
    0, /* end */
    &author_output,
};

/***********************************************************************
 * version_output : output version cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
version_output(struct cnode *cnode, unsigned int indent)
{
    printf("%*s<version>\n", indent, "");
    outputchildren(cnode, indent, 1);
    printf("%*s</version>\n", indent, "");
}

/***********************************************************************
 * cnode type version
 */
static const struct cnodefuncs version_funcs = {
    0, /* !indesc */
    0, /* !needpara */
    &default_askend,
    0, /* end */
    &version_output,
};

/***********************************************************************
 * cnode type code
 */
/***********************************************************************
 * code_end : end a code cnode
 *
 * Enter:   cnode struct
 */
static void
code_end(struct cnode *cnode)
{
    if (incode) {
        /* The incode flag has not been cleared, so this code cnode is
         * being ended implicitly. We complain about that. */
        locerrorexit(cnode->filename, cnode->linenum, "mismatched \\code");
    }
}

/***********************************************************************
 * code_output : output code cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
code_output(struct cnode *cnode, unsigned int indent)
{
    /* Note capitalization to differentiate it from HTML code element. */
    if(cnode->attrtext)
	    printf("%*s<Code %s>", indent, "", cnode->attrtext);
	else
	    printf("%*s<Code>", indent, "");
    outputchildren(cnode, indent, 1);
    printf("</Code>\n");
}

static const struct cnodefuncs code_funcs = {
    0, /* indesc */
    0, /* !needpara */
    &default_askend,
    &code_end, /* end */
    &code_output,
};

/***********************************************************************
 * startpara : start a new para cnode in the parse tree
 *
 * Enter:   cnode = current cnode
 *          type = vtable for particular type of cnode
 *
 * Return:  new current cnode
 */
static struct cnode *
startpara(struct cnode *cnode, const struct cnodefuncs *type)
{
    struct cnode *newcnode;
    while ((*cnode->funcs->askend)(cnode, type))
        cnode = endcnode(cnode);
    newcnode = memalloc(sizeof(struct cnode));
    newcnode->funcs = type;
    return startcnode(cnode, newcnode);
}

/***********************************************************************
 * cnode type text
 */
struct textcnode {
    struct cnode cn;
    unsigned char *data;
    unsigned int len;
    unsigned int max;
};

/***********************************************************************
 * text_end : end a text cnode
 *
 * Enter:   cnode struct
 */
static void
text_end(struct cnode *cnode)
{
    struct textcnode *textcnode = (void *)cnode;
    textcnode->data[textcnode->len] = 0;
    textcnode->max = textcnode->len + 1;
    textcnode->data = memrealloc(textcnode->data, textcnode->max);
}

/***********************************************************************
 * text_output : output text cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
text_output(struct cnode *cnode, unsigned int indent)
{
    /* We do not indent, in case this is inside a code cnode. */
    struct textcnode *textcnode = (void *)cnode;
    unsigned int len = textcnode->len;
    unsigned const char *p = textcnode->data;
    while (len) {
        unsigned int thislen;
        const char *thisptr;
        thislen = p[0];
        /* (void *) cast is to avoid a warning from the MS compiler.
         * I think the warning is wrong, but I could be wrong. */
        memcpy((void *)&thisptr, p + 1, sizeof(void *));
        p += 1 + sizeof(void *);
        len -= 1 + sizeof(void *);
        printtext(thisptr, thislen, 0);
    }
}

static const struct cnodefuncs text_funcs = {
    1, /* !indesc */
    0, /* !needpara */
    &default_askend,
    &text_end, /* end */
    &text_output,
};

/***********************************************************************
 * cnode type html (HTML element)
 */
struct htmlcnode {
    struct cnode cn;
    const struct htmleldesc *desc;
    char attrs[1];
};

/***********************************************************************
 * html_end : end an html cnode
 *
 * Enter:   cnode struct
 */
static void
html_end(struct cnode *cnode)
{
    if (((struct htmlcnode *)cnode)->desc->flags & HTMLEL_BLOCK)
        inhtmlblock--;
}

/***********************************************************************
 * html_output : output html cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
html_output(struct cnode *cnode, unsigned int indent)
{
    struct htmlcnode *htmlcnode = (void *)cnode;
    if (!(htmlcnode->desc->flags & HTMLEL_INLINE))
        printf("%*s", indent, "");
    if (htmlcnode->cn.children) {
        printf("<%s%s>", htmlcnode->desc->name, htmlcnode->attrs);
        if (!(htmlcnode->desc->flags & HTMLEL_INLINE))
            putchar('\n');
        outputchildren(&htmlcnode->cn, indent, 1);
        if (!(htmlcnode->desc->flags & HTMLEL_INLINE))
            printf("%*s", indent, "");
        printf("</%s>", htmlcnode->desc->name);
    } else
        printf("<%s%s/>", htmlcnode->desc->name, htmlcnode->attrs);
    if (!(htmlcnode->desc->flags & HTMLEL_INLINE))
        putchar('\n');
}

static const struct cnodefuncs html_funcs = {
    1, /* indesc */
    0, /* !needpara */
    &default_askend,
    &html_end, /* end */
    &html_output,
};

/***********************************************************************
 * starthtmlcnode : start a new html cnode
 *
 * Enter:   cnode = current cnode
 *          htmleldesc = html element descriptor
 *          attrs = attributes text
 *          attrslen = length of attributes text
 *          filename
 *          linenum = line number
 *
 * Return:  new current cnode
 */
static struct cnode *
starthtmlcnode(struct cnode *cnode, const struct htmleldesc *htmleldesc,
               const char *attrs, unsigned int attrslen,
               const char *filename, unsigned int linenum)
{
    struct htmlcnode *htmlcnode;
    /* First close enough elements to get to a content
     * model that will accept this new element. */
    for (;;) {
        if (cnode->funcs != &html_funcs) {
            /* Not in any html element. We can accept any block element
             * (in which case we need to close the current paragraph
             * first) or any inline element (in which case we need to
             * close the current text cnode first). */
            if (!(htmleldesc->flags & HTMLEL_INLINE)) {
                if (!(htmleldesc->flags & HTMLEL_BLOCK))
                    locerrorexit(filename, linenum, "<%s> not valid here", htmleldesc->name);
                while ((*cnode->funcs->askend)(cnode, 0))
                    cnode = endcnode(cnode);
            } else {
                while (cnode->funcs == &text_funcs)
                    cnode = endcnode(cnode);
            }
            break;
        }
        htmlcnode = (struct htmlcnode *)cnode;
        if (!(htmleldesc->flags & htmlcnode->desc->content))
            locerrorexit(filename, linenum, "<%s> not valid here", htmleldesc->name);
        break;
    }
    if (htmleldesc->flags & HTMLEL_BLOCK)
        inhtmlblock++;
    /* Create the new html cnode. */
    htmlcnode = memalloc(sizeof(struct htmlcnode) + attrslen);
    htmlcnode->desc = htmleldesc;
    htmlcnode->cn.funcs = &html_funcs;
    htmlcnode->cn.filename = filename;
    htmlcnode->cn.linenum = linenum;
    memcpy(htmlcnode->attrs, attrs, attrslen);
    htmlcnode->attrs[attrslen] = 0;
    /* Start the html cnode. */
    cnode = startcnode(cnode, &htmlcnode->cn);
    return cnode;
}

/***********************************************************************
 * param_output : output param cnode
 *
 * Enter:   cnode for param
 *          indent = indent (nesting) level
 *
 * This is only used for a \param inside a \def-device-cap. A normal
 * \param that gets attached to a function argument gets changed to
 * a \return so it does not use this output function.
 */
static void
param_output(struct cnode *cnode, unsigned int indent)
{
    struct paramcnode *paramcnode = (void *)cnode;
    printf("%*s<param identifier=\"%s\">\n", indent, "", paramcnode->name);
    outputchildren(&paramcnode->cn, indent, 1);
    printf("%*s</param>\n", indent, "");
}

/***********************************************************************
 * cnode type param
 */
static const struct cnodefuncs param_funcs = {
    0, /* !indesc */
    0, /* !needpara */
    &default_askend,
    0, /* end */
    &param_output,
};

/***********************************************************************
 * cnode type throw
 */
static const struct cnodefuncs throw_funcs = {
    0, /* !indesc */
    0, /* !needpara */
    &default_askend,
    0, /* end */
    &return_output,
};

/***********************************************************************
 * startparamcnode : start param (or throw) cnode
 *
 * Enter:   cnode = current cnode
 *          word = name of param
 *          wordlen = length of name
 *          inout = bit 0 = in, bit 1 = out
 *          type = &param_funcs or &throw_funcs
 *
 * Return:  new current cnode
 */
static struct cnode *
startparamcnode(struct cnode *cnode, const char *word, unsigned int wordlen,
                int inout, const struct cnodefuncs *funcs)
{
    struct paramcnode *paramcnode;
    paramcnode = memalloc(sizeof(struct paramcnode) + wordlen);
    paramcnode->cn.funcs = funcs;
    memcpy(paramcnode->name, word, wordlen);
    paramcnode->name[wordlen] = 0;
    paramcnode->inout = inout;
    return startcnode(cnode, &paramcnode->cn);
}

/***********************************************************************
 * api_feature_output : output api-feature cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
api_feature_output(struct cnode *cnode, unsigned int indent)
{
    struct paramcnode *paramcnode = (void *)cnode;
    printf("%*s<api-feature identifier=\"%s\">\n", indent, "", paramcnode->name);
    outputchildren(cnode, indent, 1);
    printf("%*s</api-feature>\n", indent, "");
}

/***********************************************************************
 * cnode type api_feature
 */
static const struct cnodefuncs api_feature_funcs = {
    0, /* !indesc */
    0, /* needpara */
    &default_askend,
    0, /* end */
    &api_feature_output,
};

/***********************************************************************
 * device_cap_output : output device-cap cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
device_cap_output(struct cnode *cnode, unsigned int indent)
{
    struct paramcnode *paramcnode = (void *)cnode;
    printf("%*s<device-cap identifier=\"%s\">\n", indent, "", paramcnode->name);
    outputchildren(cnode, indent, 1);
    printf("%*s</device-cap>\n", indent, "");
}

/***********************************************************************
 * cnode type device_cap
 */
static const struct cnodefuncs device_cap_funcs = {
    0, /* !indesc */
    0, /* needpara */
    &default_askend,
    0, /* end */
    &device_cap_output,
};

/***********************************************************************
 * def_api_feature_askend : ask if def-api-feature cnode wants to end at new para
 *
 * Enter:   cnode for def-api-feature
 *          type = cnodefuncs for new para (0 if html block element)
 *
 * Return:  non-zero to end the def-api-feature
 */
static int
def_api_feature_askend(struct cnode *cnode, const struct cnodefuncs *type)
{
    /* A def-api-feature does not end at a plain para, an html block element,
     * a brief para, or a device-cap. */
    if (!type || type == &para_funcs || type == &device_cap_funcs || type == &brief_funcs)
        return 0;
    return 1;
}

/***********************************************************************
 * def_api_feature_output : output def-api-feature cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
def_api_feature_output(struct cnode *cnode, unsigned int indent)
{
    struct paramcnode *paramcnode = (void *)cnode;
    printf("%*s<def-api-feature identifier=\"%s\">\n", indent, "", paramcnode->name);
    printf("%*s<descriptive>\n", indent + 2, "");
    outputchildren(cnode, indent + 2, 0);
    printf("%*s</descriptive>\n", indent + 2, "");
    printf("%*s</def-api-feature>\n", indent, "");
}

/***********************************************************************
 * cnode type def_api_feature
 */
static const struct cnodefuncs def_api_feature_funcs = {
    0, /* !indesc */
    1, /* needpara */
    &def_api_feature_askend,
    0, /* end */
    &def_api_feature_output,
};

/***********************************************************************
 * def_api_feature_set_askend : ask if def-api-feature-set cnode wants to end at new para
 *
 * Enter:   cnode for def-api-feature-set
 *          type = cnodefuncs for new para (0 if html block element)
 *
 * Return:  non-zero to end the def-api-feature-set
 */
static int
def_api_feature_set_askend(struct cnode *cnode, const struct cnodefuncs *type)
{
    /* A def-api-feature-set does not end at a plain para, an html block element,
     * a brief para, or an api-feature. */
    if (!type || type == &para_funcs || type == &api_feature_funcs || type == &brief_funcs)
        return 0;
    return 1;
}

/***********************************************************************
 * def_api_feature_set_output : output def-api-feature-set cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
def_api_feature_set_output(struct cnode *cnode, unsigned int indent)
{
    struct paramcnode *paramcnode = (void *)cnode;
    printf("%*s<def-api-feature-set identifier=\"%s\">\n", indent, "", paramcnode->name);
    printf("%*s<descriptive>\n", indent + 2, "");
    outputchildren(cnode, indent + 2, 0);
    printf("%*s</descriptive>\n", indent + 2, "");
    printf("%*s</def-api-feature-set>\n", indent, "");
}

/***********************************************************************
 * cnode type def_api_feature_set
 */
static const struct cnodefuncs def_api_feature_set_funcs = {
    0, /* !indesc */
    1, /* needpara */
    &def_api_feature_set_askend,
    0, /* end */
    &def_api_feature_set_output,
};

/***********************************************************************
 * def_instantiated_askend : ask if def-instantiated cnode wants to end at new para
 *
 * Enter:   cnode for def-instantiated
 *          type = cnodefuncs for new para (0 if html block element)
 *
 * Return:  non-zero to end the def-instantiated
 */
static int
def_instantiated_askend(struct cnode *cnode, const struct cnodefuncs *type)
{
    /* A def-instantiated does not end at a plain para, an html block element,
     * a brief para, or an api-feature. */
    if (!type || type == &para_funcs || type == &api_feature_funcs || type == &brief_funcs)
        return 0;
    return 1;
}

/***********************************************************************
 * def_instantiated_output : output def-instantiated cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
def_instantiated_output(struct cnode *cnode, unsigned int indent)
{
    printf("%*s<def-instantiated>\n", indent, "");
    printf("%*s<descriptive>\n", indent + 2, "");
    outputchildren(cnode, indent + 2, 0);
    printf("%*s</descriptive>\n", indent + 2, "");
    printf("%*s</def-instantiated>\n", indent, "");
}

/***********************************************************************
 * cnode type def_instantiated
 */
static const struct cnodefuncs def_instantiated_funcs = {
    0, /* !indesc */
    1, /* needpara */
    &def_instantiated_askend,
    0, /* end */
    &def_instantiated_output,
};

/***********************************************************************
 * def_device_cap_askend : ask if def-device-cap cnode wants to end at new para
 *
 * Enter:   cnode for def-device-cap
 *          type = cnodefuncs for new para (0 if html block element)
 *
 * Return:  non-zero to end the def-device-cap
 */
static int
def_device_cap_askend(struct cnode *cnode, const struct cnodefuncs *type)
{
    /* A def-device-cap does not end at a plain para, an html block element,
     * a \brief para, or a param. */
    if (!type || type == &para_funcs || type == &param_funcs || type == &brief_funcs)
        return 0;
    return 1;
}

/***********************************************************************
 * def_device_cap_output : output def_device-cap cnode
 *
 * Enter:   cnode for root
 *          indent = indent (nesting) level
 */
static void
def_device_cap_output(struct cnode *cnode, unsigned int indent)
{
    struct paramcnode *paramcnode = (void *)cnode;
    printf("%*s<def-device-cap identifier=\"%s\">\n", indent, "", paramcnode->name);
    printf("%*s<descriptive>\n", indent + 2, "");
    outputchildren(cnode, indent + 2, 0);
    printf("%*s</descriptive>\n", indent + 2, "");
    printf("%*s</def-device-cap>\n", indent, "");
}

/***********************************************************************
 * cnode type def_device_cap
 */
static const struct cnodefuncs def_device_cap_funcs = {
    0, /* !indesc */
    1, /* needpara */
    &def_device_cap_askend,
    0, /* end */
    &def_device_cap_output,
};

/***********************************************************************
 * addtext : add text to current text node, starting one if necessary
 *
 * Enter:   cnode = current cnode
 *          text
 *          len = length of text
 *
 * Return:  new current cnode
 */
static struct cnode *
addtext(struct cnode *cnode, const char *text, unsigned int len)
{
    struct textcnode *textcnode;
    if (!len)
        return cnode;
    if (cnode->funcs != &text_funcs) {
        /* Start new text cnode. */
        textcnode = memalloc(sizeof(struct textcnode));
        textcnode->cn.funcs = &text_funcs;
        cnode = startcnode(cnode, &textcnode->cn);
    }
    textcnode = (void *)cnode;
    do {
        unsigned char buf[1 + sizeof(void *)];
        unsigned int thislen = len;
        if (thislen > 255)
            thislen = 255;
        /* Encode a record as a single byte length followed by a pointer. */
        buf[0] = thislen;
        memcpy(buf + 1, &text, sizeof(void *));
        /* Add to the text cnode's data. */
        if (textcnode->len + sizeof(buf) >= textcnode->max) {
            /* Need to reallocate (or allocate) data buffer. */
            textcnode->max = textcnode->max ? 2 * textcnode->max : 1024;
            textcnode->data = memrealloc(textcnode->data, textcnode->max);
        }
        memcpy(textcnode->data + textcnode->len, buf, sizeof(buf));
        textcnode->len += sizeof(buf);
        text += thislen;
        len -= thislen;
    } while (len);
    return &textcnode->cn;
}

/***********************************************************************
 * iswhitespace : determine if character is whitespace
 *
 * Enter:   ch = character
 *
 * Return:  1 if whitespace
 */
static inline int
iswhitespace(int ch)
{
    unsigned int i = ch - 1;
    if (i >= 32)
        return 0;
    return 0x80001100 >> i & 1;
}

/***********************************************************************
 * parseword : parse the next word, ignoring leading whitespace
 *
 * Enter:   *pp = text pointer
 *
 * Return:  0 else start of word (*pp updated to just beyond end)
 */
static const char *
parseword(const char **pp)
{
    const char *p = *pp, *word = 0;
    int ch = *p;
    while (iswhitespace(ch))
        ch = *++p;
    word = p;
    while ((unsigned)((ch & ~0x20) - 'A') <= 'Z' - 'A'
            || (unsigned)(ch - '0') < 10 || ch == '_' || ch == '.'
            || ch == ':' || ch == '/' || ch == '-')
    {
        ch = *++p;
    }
    if (p == word)
        return 0;
    *pp = p;
    return word;
}

/***********************************************************************
 * Doxygen command handlers
 *
 * Enter:   p = text just after command
 *          *pcnode = pointer to current cnode struct
 *          type = 0 else cnodefuncs pointer for type of node to start
 *          filename, linenum = current filename and line number (for error reporting)
 *
 * Return:  p = updated if extra text was eaten
 *
 * On return, *pnode is updated if any node was closed or opened.
 */

/***********************************************************************
 * Doxygen command handler : \b
 */
static const char *
dox_b(const char *p, struct cnode **pcnode, const struct cnodefuncs *type,
      const char *filename, unsigned int linenum, const char *cmdname)
{
    struct cnode *cnode = *pcnode;
    const char *word = parseword(&p);
    /* Silently ignore \b with no following word. */
    if (word) {
        struct cnode *mycnode;
        mycnode = cnode = starthtmlcnode(cnode, HTMLELDESC_B, 0, 0, filename, linenum);
        cnode = addtext(cnode, word, p - word);
        while (cnode != mycnode)
            cnode = endcnode(cnode);
        cnode = endcnode(cnode);
    }
    *pcnode = cnode;
    return p;
}

/***********************************************************************
 * Doxygen command handler : \n
 */
static const char *
dox_n(const char *p, struct cnode **pcnode, const struct cnodefuncs *type,
      const char *filename, unsigned int linenum, const char *cmdname)
{
    struct cnode *cnode = *pcnode;
    cnode = starthtmlcnode(cnode, HTMLELDESC_BR, 0, 0, filename, linenum);
    cnode = endcnode(cnode);
    *pcnode = cnode;
    return p;
}

/***********************************************************************
 * Doxygen command handler : \code
 */
static const char *
dox_code(const char *p, struct cnode **pcnode, const struct cnodefuncs *type,
         const char *filename, unsigned int linenum, const char *cmdname)
{
    *pcnode = startpara(*pcnode, &code_funcs);
    (*pcnode)->filename = filename;
    (*pcnode)->linenum = linenum; /* for reporting mismatched \code error */
    incode = 1;
    return p;
}

/***********************************************************************
 * Doxygen command handler : \endcode
 */
static const char *
dox_endcode(const char *p, struct cnode **pcnode, const struct cnodefuncs *type,
            const char *filename, unsigned int linenum, const char *cmdname)
{
    incode = 0;
    *pcnode = endspecificcnode(*pcnode, &code_funcs, filename, linenum);
    return p;
}

/***********************************************************************
 * Doxygen command handler : \param
 */
static const char *
dox_param(const char *p, struct cnode **pcnode, const struct cnodefuncs *type,
          const char *filename, unsigned int linenum, const char *cmdname)
{
    struct cnode *cnode = *pcnode;
    unsigned int inout = 0;
    const char *word;
    /* Check for "in", "out" or both as attributes. */
    if (*p == '[') {
        for (;;) {
            p++;
            if (!memcmp(p, "in", 2)) {
                inout |= 1;
                p += 2;
            } else if (!memcmp(p, "out", 3)) {
                inout |= 2;
                p += 3;
            } else
                break;
            if (*p != ',')
                break;
        }
        if (*p != ']')
            locerrorexit(filename, linenum, "bad attributes on \\param");
        p++;
    }
    /* Get the next word as the parameter name. */
    word = parseword(&p);
    if (!word)
        locerrorexit(filename, linenum, "expected word after \\param");
    /* Close any open nodes. */
    while ((*cnode->funcs->askend)(cnode, type))
        cnode = endcnode(cnode);
    /* Create a new param cnode. */
    cnode = startparamcnode(cnode, word, p - word, inout, type);
    cnode->filename = filename;
    cnode->linenum = linenum;
    *pcnode = cnode;
    return p;
}

/***********************************************************************
 * Doxygen command handler : \brief, \return
 */
static const char *
dox_para(const char *p, struct cnode **pcnode, const struct cnodefuncs *type,
         const char *filename, unsigned int linenum, const char *cmdname)
{
    *pcnode = startpara(*pcnode, type);
    return p;
}

/***********************************************************************
 * Doxygen command handler : \throw
 */
static const char *
dox_throw(const char *p, struct cnode **pcnode, const struct cnodefuncs *type,
          const char *filename, unsigned int linenum, const char *cmdname)
{
    struct cnode *cnode = *pcnode;
    const char *word;
    /* Get the next word as the exception name. */
    word = parseword(&p);
    if (!word)
        locerrorexit(filename, linenum, "expected word after \\throw");
    /* Close any open nodes. */
    while ((*cnode->funcs->askend)(cnode, type))
        cnode = endcnode(cnode);
    /* Create a new throw cnode. */
    cnode = startparamcnode(cnode, word, p - word, 0, type);
    cnode->filename = filename;
    cnode->linenum = linenum;
    *pcnode = cnode;
    return p;
}

/***********************************************************************
 * Doxygen command handler : \lang
 */
static const char *
dox_attr(const char *p, struct cnode **pcnode, const struct cnodefuncs *type,
          const char *filename, unsigned int linenum, const char *cmdname)
{
  struct cnode *cnode = *pcnode;
    const char *word;
    int len, wordlen, offset = 0;
	char *attrtext;
    /* Get the next word as the attribute value. */
    word = parseword(&p);
    if (!word)
        locerrorexit(filename, linenum, "expected word after \\%s", cmdname);

	len = strlen(cmdname) + (wordlen = p-word) + 4; /* p="word"\0 */
	if(cnode->attrtext)
	  len += (offset = strlen(cnode->attrtext)) + 1; /* add space for space */
	attrtext = memalloc(len);
	if(offset) {
		memcpy(attrtext, cnode->attrtext, offset);
		attrtext[offset++] = ' ';
		memfree(((void*)cnode->attrtext));
	}
	offset += sprintf(&attrtext[offset], "%s=\"", cmdname);
	memcpy(&attrtext[offset], word, wordlen);
	strcpy(&attrtext[offset + wordlen], "\"");
	cnode->attrtext = attrtext;
	/* skip delimiter because it won't be done otherwise */
	if(incode && iswhitespace(*p)) ++p;
    return p;
}

/***********************************************************************
 * commands : table of Doxygen commands
 */
struct command {
    const char *(*func)(const char *p, struct cnode **pcnode, const struct cnodefuncs *type, const char *filename, unsigned int linenum, const char *cmdname);
    const struct cnodefuncs *type;
    unsigned int namelen;
    const char *name;
};
static const struct command commands[] = {
    { &dox_throw, &def_api_feature_funcs, 15, "def-api-feature" },
    { &dox_throw, &def_api_feature_set_funcs, 19, "def-api-feature-set" },
    { &dox_para, &def_instantiated_funcs, 16, "def-instantiated" },
    { &dox_para, &name_funcs, 4, "name" },
    { &dox_para, &author_funcs, 6, "author" },
    { &dox_b, 0, 1, "b" },
    { &dox_para, &brief_funcs, 5, "brief" },
    { &dox_code, 0, 4, "code" },
    { &dox_throw, &def_device_cap_funcs, 14, "def-device-cap" },
    { &dox_attr, 0, 4, "lang" },
    { &dox_endcode, 0, 7, "endcode" },
    { &dox_n, 0, 1, "n" },
    { &dox_param, &param_funcs, 5, "param" },
    { &dox_para, &return_funcs, 6, "return" },
    { &dox_throw, &throw_funcs, 5, "throw" },
    { &dox_throw, &api_feature_funcs, 11, "api-feature" },
    { &dox_throw, &device_cap_funcs, 10, "device-cap" },
    { &dox_para, &version_funcs, 7, "version" },
    { 0, 0, 0 }
};

/***********************************************************************
 * parsehtmltag : parse html tag
 *
 * Enter:   start = start of tag, the '<' char
 *          *pcnode = current cnode
 *          filename = filename
 *          *plinenum = current line number
 *
 * Return:  just after the tag
 *          *pcnode and *plinenum updated if applicable
 */
static const char *
parsehtmltag(const char *start, struct cnode **pcnode,
             const char *filename, unsigned int *plinenum)
{
    struct cnode *cnode = *pcnode;
    const char *end = start + 1, *endname = 0, *name = end;
    int ch = *end;
    int quote = 0;
    int close = 0;
    unsigned int linenum = *plinenum;
    const struct htmleldesc *htmleldesc;
    if (ch == '/') {
        close = 1;
        ch = *++end;
        name = end;
    }
    /* Find the end of the tag. */
    for (;;) {
        if (!ch)
            locerrorexit(filename, *plinenum, "unterminated HTML tag");
        if (ch == '\n')
            linenum++;
        else if (iswhitespace(ch) || ch == '/') {
            if (!endname)
                endname = end;
        } else if (!quote) {
            if (ch == '"' || ch == '\'')
                quote = ch;
            else if (ch == '>')
                break;
        } else {
            if (ch == quote)
                quote = 0;
        }
        ch = *++end;
    }
    if (!endname)
        endname = end;
    end++;
    /* See if it's an xml open-close tag. */
    if (!close && endname != name && end[-2] == '/')
        close = 2;
    /* Find the tag from our list. */
    htmleldesc = htmleldescs;
    for (;;) {
        if (!htmleldesc->namelen) {
            locerrorexit(filename, *plinenum, "unrecognized HTML tag %.*s",
                    end - start, start);
        }
        if (htmleldesc->namelen == endname - name
                && !strncasecmp(htmleldesc->name, name, endname - name))
        {
            break;
        }
        htmleldesc++;
    }
    if (close == 1) {
        /* Closing tag. Find open element to close. */
        for (;;) {
            struct htmlcnode *htmlcnode;
            if (cnode->funcs != &text_funcs) {
                if (cnode->funcs != &html_funcs) {
                    locerrorexit(filename, *plinenum, "mismatched %.*s",
                            end - start, start);
                }
                htmlcnode = (struct htmlcnode *)cnode;
                if (htmlcnode->desc == htmleldesc)
                    break;
                if (!(htmlcnode->desc->flags & HTMLEL_AUTOCLOSE)) {
                    locerrorexit(filename, htmlcnode->cn.linenum,
                            "mismatched <%.*s>",
                            htmlcnode->desc->namelen, htmlcnode->desc->name);
                }
            }
            cnode = endcnode(cnode);
        }
        cnode = endcnode(cnode);
    } else {
        /* Opening tag. */
      if (close !=2)
	   cnode = starthtmlcnode(cnode, htmleldesc, endname, end - 1 - endname, filename, *plinenum);
      else // don't include the closing "/" in the attributes list
	   cnode = starthtmlcnode(cnode, htmleldesc, endname, end - 2 - endname, filename, *plinenum);
      if (close == 2 || (htmleldesc->content & HTMLEL_EMPTY)) {
	/* Empty element -- close it again. */
	cnode = endcnode(cnode);
      }
    }
    *pcnode = cnode;
    *plinenum = linenum;
    return end;
}

/***********************************************************************
 * parsecomment : parse one comment
 *
 * Enter:   comment struct
 */
static void
parsecomment(struct comment *comment)
{
    struct cnode *cnode = &comment->root;
    const char *p = comment->text + comment->back;
    unsigned int linenum = comment->linenum - 1;
    int ch;
    curcomment = comment;
    incode = 0;
    inhtmlblock = 0;
    cnode->funcs = &root_funcs;
    for (;;) {
        /* Start of new line. */
        const char *starttext;
        ch = *p;
        linenum++;
        {
            /* Find first non-whitespace character. */
            const char *p2 = p;
            int ch2 = ch;
            while (iswhitespace(ch2))
                ch2 = *++p2;
            if (comment->type == TOK_BLOCKCOMMENT && ch2 == '*') {
                /* Ignore initial * in block comment (even in \code block). */
                ch2 = *++p2;
                ch = ch2;
                p = p2;
                if (ch == '*')
                    goto checkforlineofstars;
                while (iswhitespace(ch2))
                    ch2 = *++p2;
            }
            if (comment->type == TOK_INLINECOMMENT && ch2 == '/') {
checkforlineofstars:
                if (!incode) {
                    /* Ignore whole line of * for block comment or / for inline
                     * comment if that is the only thing on the line. */
                    const char *p3 = p2;
                    int ch3;
                    do ch3 = *++p3; while (ch3 == ch2);
                    while (iswhitespace(ch3)) ch3 = *++p3;
                    if (!ch3 || ch3 == '\n') {
                        /* Reached end of line (or whole comment) -- treat as
                         * empty line. */
                        ch2 = ch3;
                        p2 = p3;
                    }
                }
            }
            if (!incode) {
                /* Only allow whitespace omission above to take effect if
                 * not in \code block. */
                ch = ch2;
                p = p2;
            }
        }
        if (!ch) {
            /* End of comments -- finish. */
            break;
        }
        if (!incode && !inhtmlblock && ch == '\n') {
            /* Blank line -- finish any para, but only if not in code and
             * not in any HTML block element. */
            while ((*cnode->funcs->askend)(cnode, 0))
                cnode = endcnode(cnode);
            p++;
            continue;
        }
        /* Start new para if there isn't already one going. */
        if (cnode->funcs->needpara)
            cnode = startpara(cnode, &para_funcs);
        /* Process text on the line. */
        starttext = p;
        while (ch && ch != '\n') {
            if (ch != '\\' && ch != '<' /* && ch != '@' */ && ch != '$'
                    && ch != '&' && ch != '\r')
            {
                ch = *++p;
                continue;
            }
            /* Output any pending text. */
            if (p - starttext)
                cnode = addtext(cnode, starttext, p - starttext);
	    /* Ignore \r in DOS line returns */
	    if (ch == '\r') {
	        ch = *++p;
  	        starttext = p;
		continue;
	    }
            if (ch == '$')
                locerrorexit(comment->filename, linenum, "use \\$ instead of $");
            /* See if it is an html named entity. */
            if (ch == '&' && p[1] != '#') {
                const char *entity = ENTITIES;
                /* This search could be faster if the entity names were put
                 * in a hash table or something. */
                const char *semicolon = strchr(p, ';');
                unsigned int len;
                if (!semicolon)
                    locerrorexit(comment->filename, linenum, "unterminated HTML entity");
                p++;
                for (;;) {
                    len = strlen(entity);
                    if (!len)
                        locerrorexit(comment->filename, linenum, "unrecognised HTML entity &%.*s;", semicolon - p, p);
                    if (len == semicolon - p && !memcmp(p, entity, len))
                        break;
                    entity += len + 1;
                    entity += strlen(entity) + 1;
                }
                entity += len + 1;
                cnode = addtext(cnode, entity, strlen(entity));
                p = semicolon + 1;
                ch = *p;
                starttext = p;
                continue;
            }
            /* See if it is a backslash escape sequence. */
            else if (ch == '\\') {
                const char *match = "\\@&$#<>%";
                const char *pos;
                ch = p[1];
                pos = strchr(match, ch);
                if (pos) {
                    /* Got a \ escape sequence. */
                    const char *text = 
                        "\\\0    @\0    &amp;\0$\0    #\0    &lt;\0 >\0    %"
                        + 6 * (pos - match);
                    cnode = addtext(cnode, text, strlen(text));
                    p += 2;
                    ch = *p;
                    starttext = p;
                    continue;
                }
            } else if (ch == '<') {
                if (incode) {
                    ch = *++p;
                    starttext = p;
                    continue;
                }
                /* It's an html tag. */
                p = parsehtmltag(p, &cnode, comment->filename, &linenum);
                ch = *p;
                starttext = p;
                continue;
            }
            {
                /* Got a doxygen command. First work out its length. */
                const char *start = ++p;
                unsigned int cmdlen;
                const struct command *command;
                ch = *p;
                while ((unsigned)((ch & ~0x20) - 'A') <= 'Z' - 'A'
                        || (unsigned)(ch - '0') < 10 || ch == '_' || ch == '-')
                {
                    ch = *++p;
                }
                cmdlen = p - start;
                if (!cmdlen)
                    locerrorexit(comment->filename, linenum, "\\ or @ without Doxygen command");
                /* Look it up in the table. */
                command = commands;
                for (;;) {
                    if (!command->namelen) {
                        locerrorexit(comment->filename, linenum, "unrecognized Doxygen command '%.*s'",
                                cmdlen + 1, start - 1);
                    }
                    if (command->namelen == cmdlen
                            && !memcmp(command->name, start, cmdlen))
                    {
                        break;
                    }
                    command++;
                }
                p = (*command->func)(p, &cnode, command->type,
                        comment->filename, linenum, command->name);
                ch = *p;
                starttext = p;
            }
        }
        if (p - starttext) {
            /* Start new para if there isn't already one going. */
            if (cnode->funcs->needpara)
                cnode = startpara(cnode, &para_funcs);
            cnode = addtext(cnode, starttext, p - starttext);
        }
        if (!ch)
            break;
        if (cnode->funcs == &text_funcs)
            addtext(cnode, "\n", 1);
        p++;
    }
    /* Finish the root cnode. */
    do
        cnode = endcnode(cnode);
    while (cnode);
    assert(!incode);
    assert(!inhtmlblock);
}

/***********************************************************************
 * parsecomments : parse comments
 *
 * Enter:   comment = first comment in list
 */
static void
parsecomments(struct comment *comment)
{
    while (comment) {
        parsecomment(comment);
        comment = comment->next;
    }
}

/***********************************************************************
 * attachcommenttonode : attach comment struct to node
 *
 * Enter:   node = parse node for identifier
 *          comment = comment struct
 */
static void
attachcommenttonode(struct node *node, struct comment *comment)
{
    comment->next = node->comments;
    node->comments = comment;
}

/***********************************************************************
 * attachcomments : attach comments to applicable parse nodes
 *
 * Enter:   comment = first in (reversed) list of comment structs
 *          root = root parse node (for attaching \file comment blocks to)
 */
static void
attachcomments(struct comment *comment, struct node *root)
{
    while (comment) {
        struct comment *next = comment->next;
        /* See if there are any \param, \return, \throw, \def-api-feature or
         * \def-device-cap cnodes to detach and attach
         * elsewhere. (This only looks at top-level nodes, direct children
         * of the root, so does not detach a \param inside a
         * \def-device-cap.) */
        struct cnode **pcnode = &comment->root.children;
        for (;;) {
            struct cnode *cnode = *pcnode;
            if (!cnode)
                break;
            if (cnode->funcs == &param_funcs || cnode->funcs == &return_funcs
                    || cnode->funcs == &throw_funcs)
            {
                /* Found a \param or \return or \throw to detach. Find the
                 * parameter/exception of the same name, or the return type. */
                struct node *node;
                struct comment *newcomment;
                if (cnode->funcs == &param_funcs) {
                    node = findparamidentifier(comment->node,
                        ((struct paramcnode *)cnode)->name);
                    if (!node)
                        locerrorexit(comment->filename, cnode->linenum, "no parameter '%s' found", ((struct paramcnode *)cnode)->name);
                } else if (cnode->funcs == &return_funcs) {
                    node = findreturntype(comment->node);
                    if (!node)
                        locerrorexit(comment->filename, cnode->linenum, "no return type found");
                } else {
                    node = findthrowidentifier(comment->node,
                        ((struct paramcnode *)cnode)->name);
                    if (!node)
                        locerrorexit(comment->filename, cnode->linenum, "no exception '%s' found", ((struct paramcnode *)cnode)->name);
                }
                /* Detach the cnode from its old comment. */
                *pcnode = cnode->next;
                /* Create a new comment struct to contain this cnode. */
                newcomment = memalloc(sizeof(struct comment));
                newcomment->root.funcs = &root_funcs;
                newcomment->linenum = cnode->linenum;
                /* Attach the cnode. */
                newcomment->root.children = cnode;
                cnode->parent = &newcomment->root;
                cnode->next = 0;
                /* Make the cnode a \return one, just so even a \param
                 * uses return_output. */
                cnode->funcs = &return_funcs;
                /* Attach the new comment struct to the parse node. */
                attachcommenttonode(node, newcomment);
            } else {
                pcnode = &cnode->next;
            }
        }
        /* Now attach the comment to its identifier parse node. */
        {
            struct node *node = comment->node;
            if (!node)
                node = root;
            attachcommenttonode(node, comment);
        }
        comment = next;
    }
}

/***********************************************************************
 * processcomments : join, parse and attach comments
 *
 * Enter:   root = root parse node
 */
void
processcomments(struct node *root)
{
    comments = joininlinecomments(comments);
    parsecomments(comments);
    attachcomments(comments, root);
}

/***********************************************************************
 * outputdescriptive : output descriptive elements for a node
 *
 * Enter:   node = identifier node that might have some comments
 *          indent = indent (nesting) level
 */
void
outputdescriptive(struct node *node, unsigned int indent)
{
    struct comment *comment = node->comments;
    int indescriptive = 0;
    while (comment) {
        struct cnode *root = &comment->root;
        if (!indescriptive)
            printf("%*s<descriptive>\n", indent, "");
        indescriptive = 1;
        (*root->funcs->output)(root, indent + 2);
        comment = comment->next;
    }
    if (indescriptive)
        printf("%*s</descriptive>\n", indent, "");
}
