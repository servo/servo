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
#ifndef node_h
#define node_h

/* struct node : a node in the parse tree (excluding comments) */
enum { NODE_ELEMENT, NODE_ATTR, NODE_ATTRLIST };
struct node {
    int type;
    struct node *next;
    struct node *parent;
    struct node *children;
    struct comment *comments; /* list of comments attached to this node */
    /* If wsstart and end are set, they give the literal Web IDL that can
     * be output in a <webidl> element. */
    const char *wsstart;
    /* If start and end are set, they give the text of a scoped name that
     * can be enclosed in a <ref> when outputting a <webidl> element for
     * an ancestor element. */
    const char *start;
    const char *end;
    const char *id;
};

struct element {
    struct node n;
    const char *name;
};

struct attr {
    struct node n;
    const char *name;
    const char *value;
};

struct attrlist {
    struct node n;
};

struct node *newelement(const char *name);
struct node *newattr(const char *name, const char *val);
struct node *newattrlist(void);
void addnode(struct node *parent, struct node *child);
void reversechildren(struct node *node);
int nodeisempty(struct node *node);
const char *getattr(struct node *node, const char *name);
struct node *nodewalk(struct node *node);
struct node *findreturntype(struct node *node);
struct node *findparamidentifier(struct node *node, const char *name);
struct node *findthrowidentifier(struct node *node, const char *name);
void outputnode(struct node *node, unsigned int indent);

#endif /* ndef node_h */

