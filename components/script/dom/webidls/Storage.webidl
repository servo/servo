/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://html.spec.whatwg.org/multipage/#webstorage
 *
 */

interface Storage {

  readonly attribute unsigned long length;

  DOMString? key(unsigned long index);

  getter DOMString? getItem(DOMString name);

  setter creator void setItem(DOMString name, DOMString value);

  deleter void removeItem(DOMString name);

  void clear();
};
