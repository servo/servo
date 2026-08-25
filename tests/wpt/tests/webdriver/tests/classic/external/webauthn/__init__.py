# Base64url-encoded PKCS#8 EC P-256 private key
PRIVATE_KEY = (
    "MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQg8_zMDQDYAxlU-Q"
    "hk1Dwkf0v18GZca1DMF3SaJ9HPdmShRANCAASNYX5lyVCOZLzFZzrIKmeZ2jwU"
    "RmgsJYxGP__fWN_S-j5sN4tT15XEpN_7QZnt14YvI6uvAgO0uJEboFaZlOEB"
)


def create_credential(
    credential_id="Y3JlZC0x",
    is_resident_credential=False,
    rp_id="example.com",
    private_key=PRIVATE_KEY,
    sign_count=0,
    user_handle=None,
    large_blob=None,
):
    credential = {
        "credentialId": credential_id,
        "isResidentCredential": is_resident_credential,
        "rpId": rp_id,
        "privateKey": private_key,
        "signCount": sign_count,
    }
    if user_handle is not None:
        credential["userHandle"] = user_handle
    if large_blob is not None:
        credential["largeBlob"] = large_blob
    return credential
