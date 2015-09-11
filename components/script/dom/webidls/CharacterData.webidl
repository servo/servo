/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://dom.spec.whatwg.org/#characterdata
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Abstract]
interface CharacterData : Node {
  [TreatNullAs=EmptyString] attribute DOMString data;
  readonly attribute unsigned long length;
  [Throws]
  DOMString substringData(unsigned long offset, unsigned long count);
  void appendData(DOMString data);
  [Throws]
  void insertData(unsigned long offset, DOMString data);
  [Throws]
  void deleteData(unsigned long offset, unsigned long count);
  [Throws]
  void replaceData(unsigned long offset, unsigned long count, DOMString data);
};

CharacterData implements ChildNode;
CharacterData implements NonDocumentTypeChildNode;
