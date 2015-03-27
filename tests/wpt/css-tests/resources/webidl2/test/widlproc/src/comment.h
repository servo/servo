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
#ifndef comment_h
#define comment_h

struct tok;
struct node;

void addcomment(struct tok *tok);
void setcommentnode(struct node *node2);
void processcomments(struct node *root);
void outputdescriptive(struct node *node, unsigned int indent);

#endif /* ndef comment_h */
