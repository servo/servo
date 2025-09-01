/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://www.w3.org/TR/credential-management-1/#framework-credential-management
 */

// https://www.w3.org/TR/credential-management-1/#framework-credential-management
partial interface Navigator {
  [SecureContext, SameObject, Pref="dom_credential_management_enabled"] readonly attribute CredentialsContainer credentials;
};

// https://www.w3.org/TR/credential-management-1/#credentialscontainer
[Pref="dom_credential_management_enabled", Exposed=Window, SecureContext]
interface CredentialsContainer {
  [Throws] Promise<Credential?> get(optional CredentialRequestOptions options = {});
  [Throws] Promise<undefined> store(Credential credential);
  [Throws] Promise<Credential?> create(optional CredentialCreationOptions options = {});
  [Throws] Promise<undefined> preventSilentAccess();
};

// https://www.w3.org/TR/credential-management-1/#credentialrequestoptions-dictionary
dictionary CredentialRequestOptions {
  CredentialMediationRequirement mediation = "optional";
  AbortSignal signal;
    // FIXME: This should be part of a partial dictionary, but that is not implemented yet
    // From PasswordCredential.webidl
    boolean password = false;
};

// https://www.w3.org/TR/credential-management-1/#dictdef-credentialcreationoptions
dictionary CredentialCreationOptions {
  CredentialMediationRequirement mediation = "optional";
  AbortSignal signal;
  // FIXME: This should be part of a partial dictionary, but that is not implemented yet
  // From PasswordCredential.webidl
  PasswordCredentialInit password;
};

// https://www.w3.org/TR/credential-management-1/#dictdef-credentialdata
dictionary CredentialData {
  required USVString id;
};

// https://www.w3.org/TR/credential-management-1/#enumdef-credentialmediationrequirement
enum CredentialMediationRequirement {
  "silent",
  "optional",
  "conditional",
  "required"
};
