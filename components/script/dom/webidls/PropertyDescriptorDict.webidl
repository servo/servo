/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://drafts.css-houdini.org/css-properties-values-api/#registering-custom-properties
 */

// Renamed from PropertyDescriptor to avoid conflicting with a JS class of the
// same name.
dictionary PropertyDescriptorDict
{
  required DOMString name;
           DOMString syntax       = "*";
           boolean inherits       = false;
           DOMString initialValue;
};
