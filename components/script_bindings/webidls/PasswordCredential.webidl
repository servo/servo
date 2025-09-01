/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/credential-management-1/#passwordcredential-interface
 */

// https://www.w3.org/TR/credential-management-1/#passwordcredential-interface
[Pref="dom_credential_management_enabled", Exposed=Window, SecureContext]
interface PasswordCredential : Credential {
  [Throws] constructor(HTMLFormElement form);
  [Throws] constructor(PasswordCredentialData data);
  readonly attribute USVString password;
};
PasswordCredential includes CredentialUserData;

// https://www.w3.org/TR/credential-management-1/#dictdef-passwordcredentialdata
dictionary PasswordCredentialData : CredentialData {
  USVString name;
  USVString iconURL;
  required USVString origin;
  required USVString password;
};

typedef (PasswordCredentialData or HTMLFormElement) PasswordCredentialInit;
