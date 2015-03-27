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
 *
 * Node-specific functions
 ***********************************************************************/
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "comment.h"
#include "lex.h"
#include "misc.h"
#include "node.h"
#include "process.h"

struct node *
newelement(const char *name)
{
    struct element *element = memalloc(sizeof(struct element));
    element->n.type = NODE_ELEMENT;
    element->name = name;
    return &element->n;
}

struct node *
newattr(const char *name, const char *val)
{
    struct attr *attr = memalloc(sizeof(struct attr));
    attr->n.type = NODE_ATTR;
    attr->name = name;
    attr->value = val;
    return &attr->n;
}

struct node *
newattrlist(void)
{
    struct attrlist *attrlist = memalloc(sizeof(struct attrlist));
    attrlist->n.type = NODE_ATTRLIST;
    return &attrlist->n;
}

/***********************************************************************
 * addnode : add node as child of another node
 *
 * Enter:   parent node
 *          child node
 *
 * The children list is constructed backwards. This is fixed later with
 * a call to reversechildren.
 *
 * If child is an attrlist, its children are added to parent and the
 * attrlist is freed.
 */
void
addnode(struct node *parent, struct node *child)
{
    if (!child)
        return;
    if (child->type == NODE_ATTRLIST) {
        /* Add the attrs in the attrlist to parent. */
        struct node *child2;
        reversechildren(child);
        child2 = child->children;
        memfree(child);
        while (child2) {
            struct node *next = child2->next;
            addnode(parent, child2);
            child2 = next;
        }
    } else {
        child->next = parent->children;
        parent->children = child;
        child->parent = parent;
    }
}

/***********************************************************************
 * reversechildren : recursively reverse child lists
 *
 * Also sets parent field on each node.
 */
void
reversechildren(struct node *node)
{
    struct node *newlist = 0;
    struct node *child = node->children;
    while (child) {
        struct node *next = child->next;
        child->parent = node;
        child->next = newlist;
        newlist = child;
        reversechildren(child);
        child = next;
    }
    node->children = newlist;
}

/***********************************************************************
 * nodeisempty : test if node is empty (has no children)
 */
int
nodeisempty(struct node *node)
{
    return !node->children;
}

/***********************************************************************
 * nodewalk : single step of depth last traversal of node tree
 *
 * Return:  next node in walk, 0 if finished
 */
struct node *
nodewalk(struct node *node)
{
    if (node->children)
        return node->children;
    if (node->next)
        return node->next;
    do {
        node = node->parent;
        if (!node)
            return 0;
    } while (!node->next);
    return node->next;
}

/***********************************************************************
 * findchildelement : find child element of a particular name
 *
 * Enter:   node = element
 *          name = name to find
 *
 * Return:  0 else child element of that name
 */
static struct node *
findchildelement(struct node *node, const char *name)
{
    node = node->children;
    while (node) {
        if (node->type == NODE_ELEMENT) {
            struct element *element = (void *)node;
            if (!strcmp(element->name, name))
                break;
        }
        node = node->next;
    }
    return node;
}

/***********************************************************************
 * getattr : get value of attribute
 *
 * Enter:   node = element to find attribute in
 *          name = name of attribute
 *
 * Return:  0 if not found, else 0-terminated string value
 */
const char *
getattr(struct node *node, const char *name)
{
    node = node->children;
    while (node) {
        if (node->type == NODE_ATTR) {
            struct attr *attr = (void *)node;
            if (!strcmp(attr->name, name))
                return attr->value;
        }
        node = node->next;
    }
    return 0;
}

/***********************************************************************
 * findchildelementwithnameattr : find child element with a name attribute
 *                                of a particular value
 *
 * Enter:   node = element
 *          name = name to find
 *
 * Return:  0 else child element with name attr of that value
 */
static struct node *
findchildelementwithnameattr(struct node *node, const char *name)
{
    node = node->children;
    while (node) {
        if (node->type == NODE_ELEMENT) {
            const char *s = getattr(node, "name");
            if (s && !strcmp(s, name))
                break;
        }
        node = node->next;
    }
    return node;
}

/***********************************************************************
 * findreturntype : find Type parse node for return type
 *
 * Enter:   node = Operation element
 *
 * Return:  0 if not found, else Type parse node for return type
 */
struct node *
findreturntype(struct node *node)
{
    return findchildelement(node, "Type");
}

/***********************************************************************
 * findparamidentifier : find identifier parse node for parameter
 *
 * Enter:   node = Operation element
 *          name = parameter name to find
 *
 * Return:  0 if not found, else node struct for parameter identifier
 */
struct node *
findparamidentifier(struct node *node, const char *name)
{
    node = findchildelement(node, "ArgumentList");
    if (node)
        node = findchildelementwithnameattr(node, name);
    return node;
}

/***********************************************************************
 * findthrowidentifier : find identifier parse node for exception name
 *
 * Enter:   node = Operation element
 *          name = exception name to find
 *
 * Return:  0 if not found, else node for Name element, child of Raises
 *              or SetRaises
 */
struct node *
findthrowidentifier(struct node *node, const char *name)
{
    struct node *node2 = findchildelement(node, "Raises");
    if (node2)
        node2 = findchildelementwithnameattr(node2, name);
    if (!node2) {
        node2 = findchildelement(node, "SetRaises");
        if (node2)
            node2 = findchildelementwithnameattr(node2, name);
    }
    return node2;
}

/***********************************************************************
 * outputid : output the id of a node
 */
static void
outputid(struct node *node)
{
    if (node->parent)
        outputid(node->parent);
    if (node->id) {
        fputs("::", stdout);
        printtext(node->id, strlen(node->id), 1);
    }
}

/***********************************************************************
 * outputnode : output node and its children
 *
 * Enter:   node = node to output, assumed to be an element
 *          indent
 */
void
outputnode(struct node *node, unsigned int indent)
{
    struct element *element = (void *)node;
    struct node *child;
    int empty = 1;
    printf("%*s<%s", indent, "", element->name);
    child = element->n.children;
    while (child) {
        switch(child->type) {
        case NODE_ELEMENT:
            empty = 0;
            break;
        case NODE_ATTR:
            {
                struct attr *attr = (void *)child;
                printf(" %s=\"", attr->name);
                printtext(attr->value, strlen(attr->value), 1);
                printf("\"");
            }
            break;
        }
        child = child->next;
    }
    if (node->id) {
        printf(" id=\"");
        outputid(node);
        printf("\"");
    }
    if (!empty || node->comments || node->wsstart) {
        printf(">\n");
        if (node->wsstart) {
            printf("%*s  <webidl>", indent, "");
            outputwidl(node);
            printf("</webidl>\n");
        }
        outputdescriptive(node, indent + 2);
        child = element->n.children;
        while (child) {
            switch(child->type) {
            case NODE_ELEMENT:
                outputnode(child, indent + 2);
                break;
            }
            child = child->next;
        }
        printf("%*s</%s>\n", indent, "", element->name);
    } else
        printf("/>\n");
}


