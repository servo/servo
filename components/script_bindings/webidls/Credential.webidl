/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/credential-management-1/#the-credential-interface
 */

// https://www.w3.org/TR/credential-management-1/#credential
[Pref="dom_credential_management_enabled", Exposed=Window, SecureContext]
interface Credential {
  readonly attribute USVString id;
  readonly attribute DOMString type;
  static Promise<boolean> isConditionalMediationAvailable();
  static Promise<undefined> willRequestConditionalCreation();
};

// https://www.w3.org/TR/credential-management-1/#credentialuserdata
[SecureContext]
interface mixin CredentialUserData {
  // TODO: seems like interface mixins are broken: these should be optional by default?
  readonly attribute USVString? name;
  readonly attribute USVString? iconURL;
};
