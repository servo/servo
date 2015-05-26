/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Copyright (c) 2012 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// This file intentionally does not have header guards, it's included
// inside a macro to generate enum values.

// This file contains the list of network errors.

//
// Ranges:
//     0- 99 System related errors
//   100-199 Connection related errors
//   200-299 Certificate errors
//   300-399 HTTP errors
//   400-499 Cache errors
//   500-599 ?
//   600-699 FTP errors
//   700-799 Certificate manager errors
//   800-899 DNS resolver errors

#[allow(dead_code, non_camel_case_types)]
pub enum NetError {
    // An asynchronous IO operation is not yet complete.  This usually does not
    // indicate a fatal error.  Typically this error will be generated as a
    // notification to wait for some external notification that the IO operation
    // finally completed.
    IO_PENDING = 1,

    // A generic failure occurred.
    FAILED = 2,

    // An operation was aborted (due to user action,.
    ABORTED = 3,

    // An argument to the function is incorrect.
    INVALID_ARGUMENT = 4,

    // The handle or file descriptor is invalid.
    INVALID_HANDLE = 5,

    // The file or directory cannot be found.
    FILE_NOT_FOUND = 6,

    // An operation timed out.
    TIMED_OUT = 7,

    // The file is too large.
    FILE_TOO_BIG = 8,

    // An unexpected error.  This may be caused by a programming mistake or an
    // invalid assumption.
    UNEXPECTED = 9,

    // Permission to access a resource = other than the network = was denied.
    ACCESS_DENIED = 10,

    // The operation failed because of unimplemented functionality.
    NOT_IMPLEMENTED = 11,

    // There were not enough resources to complete the operation.
    INSUFFICIENT_RESOURCES = 12,

    // Memory allocation failed.
    OUT_OF_MEMORY = 13,

    // The file upload failed because the file's modification time was different
    // from the expectation.
    UPLOAD_FILE_CHANGED = 14,

    // The socket is not connected.
    SOCKET_NOT_CONNECTED = 15,

    // The file already exists.
    FILE_EXISTS = 16,

    // The path or file name is too long.
    FILE_PATH_TOO_LONG = 17,

    // Not enough room left on the disk.
    FILE_NO_SPACE = 18,

    // The file has a virus.
    FILE_VIRUS_INFECTED = 19,

    // The client chose to block the request.
    BLOCKED_BY_CLIENT = 20,

    // The network changed.
    NETWORK_CHANGED = 21,

    // The request was blocked by the URL blacklist configured by the domain
    // administrator.
    BLOCKED_BY_ADMINISTRATOR = 22,

    // The socket is already connected.
    SOCKET_IS_CONNECTED = 23,

    // The request was blocked because the forced reenrollment check is still
    // pending. This error can only occur on ChromeOS.
    // The error can be emitted by code in chrome/browser/policy/policy_helpers.cc.
    BLOCKED_ENROLLMENT_CHECK_PENDING = 24,

    // The upload failed because the upload stream needed to be re-read = due to a
    // retry or a redirect = but the upload stream doesn't support that operation.
    UPLOAD_STREAM_REWIND_NOT_SUPPORTED = 25,

    // A connection was closed (corresponding to a TCP FIN,.
    CONNECTION_CLOSED = 100,

    // A connection was reset (corresponding to a TCP RST,.
    CONNECTION_RESET = 101,

    // A connection attempt was refused.
    CONNECTION_REFUSED = 102,

    // A connection timed out as a result of not receiving an ACK for data sent.
    // This can include a FIN packet that did not get ACK'd.
    CONNECTION_ABORTED = 103,

    // A connection attempt failed.
    CONNECTION_FAILED = 104,

    // The host name could not be resolved.
    NAME_NOT_RESOLVED = 105,

    // The Internet connection has been lost.
    INTERNET_DISCONNECTED = 106,

    // An SSL protocol error occurred.
    SSL_PROTOCOL_ERROR = 107,

    // The IP address or port number is invalid (e.g. = cannot connect to the IP
    // address 0 or the port 0,.
    ADDRESS_INVALID = 108,

    // The IP address is unreachable.  This usually means that there is no route to
    // the specified host or network.
    ADDRESS_UNREACHABLE = 109,

    // The server requested a client certificate for SSL client authentication.
    SSL_CLIENT_AUTH_CERT_NEEDED = 110,

    // A tunnel connection through the proxy could not be established.
    TUNNEL_CONNECTION_FAILED = 111,

    // No SSL protocol versions are enabled.
    NO_SSL_VERSIONS_ENABLED = 112,

    // The client and server don't support a common SSL protocol version or
    // cipher suite.
    SSL_VERSION_OR_CIPHER_MISMATCH = 113,

    // The server requested a renegotiation (rehandshake,.
    SSL_RENEGOTIATION_REQUESTED = 114,

    // The proxy requested authentication (for tunnel establishment, with an
    // unsupported method.
    PROXY_AUTH_UNSUPPORTED = 115,

    // During SSL renegotiation (rehandshake, = the server sent a certificate with
    // an error.
    //
    // Note: this error is not in the -2xx range so that it won't be handled as a
    // certificate error.
    CERT_ERROR_IN_SSL_RENEGOTIATION = 116,

    // The SSL handshake failed because of a bad or missing client certificate.
    BAD_SSL_CLIENT_AUTH_CERT = 117,

    // A connection attempt timed out.
    CONNECTION_TIMED_OUT = 118,

    // There are too many pending DNS resolves = so a request in the queue was
    // aborted.
    HOST_RESOLVER_QUEUE_TOO_LARGE = 119,

    // Failed establishing a connection to the SOCKS proxy server for a target host.
    SOCKS_CONNECTION_FAILED = 120,

    // The SOCKS proxy server failed establishing connection to the target host
    // because that host is unreachable.
    SOCKS_CONNECTION_HOST_UNREACHABLE = 121,

    // The request to negotiate an alternate protocol failed.
    NPN_NEGOTIATION_FAILED = 122,

    // The peer sent an SSL no_renegotiation alert message.
    SSL_NO_RENEGOTIATION = 123,

    // Winsock sometimes reports more data written than passed.  This is probably
    // due to a broken LSP.
    WINSOCK_UNEXPECTED_WRITTEN_BYTES = 124,

    // An SSL peer sent us a fatal decompression_failure alert. This typically
    // occurs when a peer selects DEFLATE compression in the mistaken belief that
    // it supports it.
    SSL_DECOMPRESSION_FAILURE_ALERT = 125,

    // An SSL peer sent us a fatal bad_record_mac alert. This has been observed
    // from servers with buggy DEFLATE support.
    SSL_BAD_RECORD_MAC_ALERT = 126,

    // The proxy requested authentication (for tunnel establishment,.
    PROXY_AUTH_REQUESTED = 127,

    // A known TLS strict server didn't offer the renegotiation extension.
    SSL_UNSAFE_NEGOTIATION = 128,

    // The SSL server attempted to use a weak ephemeral Diffie-Hellman key.
    SSL_WEAK_SERVER_EPHEMERAL_DH_KEY = 129,

    // Could not create a connection to the proxy server. An error occurred
    // either in resolving its name = or in connecting a socket to it.
    // Note that this does NOT include failures during the actual "CONNECT" method
    // of an HTTP proxy.
    PROXY_CONNECTION_FAILED = 130,

    // A mandatory proxy configuration could not be used. Currently this means
    // that a mandatory PAC script could not be fetched = parsed or executed.
    MANDATORY_PROXY_CONFIGURATION_FAILED = 131,

    // -132 was formerly ERR_ESET_ANTI_VIRUS_SSL_INTERCEPTION

    // We've hit the max socket limit for the socket pool while preconnecting.  We
    // don't bother trying to preconnect more sockets.
    PRECONNECT_MAX_SOCKET_LIMIT = 133,

    // The permission to use the SSL client certificate's private key was denied.
    SSL_CLIENT_AUTH_PRIVATE_KEY_ACCESS_DENIED = 134,

    // The SSL client certificate has no private key.
    SSL_CLIENT_AUTH_CERT_NO_PRIVATE_KEY = 135,

    // The certificate presented by the HTTPS Proxy was invalid.
    PROXY_CERTIFICATE_INVALID = 136,

    // An error occurred when trying to do a name resolution (DNS,.
    NAME_RESOLUTION_FAILED = 137,

    // Permission to access the network was denied. This is used to distinguish
    // errors that were most likely caused by a firewall from other access denied
    // errors. See also ERR_ACCESS_DENIED.
    NETWORK_ACCESS_DENIED = 138,

    // The request throttler module cancelled this request to avoid DDOS.
    TEMPORARILY_THROTTLED = 139,

    // A request to create an SSL tunnel connection through the HTTPS proxy
    // received a non-200 (OK, and non-407 (Proxy Auth, response.  The response
    // body might include a description of why the request failed.
    HTTPS_PROXY_TUNNEL_RESPONSE = 140,

    // We were unable to sign the CertificateVerify data of an SSL client auth
    // handshake with the client certificate's private key.
    //
    // Possible causes for this include the user implicitly or explicitly
    // denying access to the private key = the private key may not be valid for
    // signing = the key may be relying on a cached handle which is no longer
    // valid = or the CSP won't allow arbitrary data to be signed.
    SSL_CLIENT_AUTH_SIGNATURE_FAILED = 141,

    // The message was too large for the transport.  (for example a UDP message
    // which exceeds size threshold,.
    MSG_TOO_BIG = 142,

    // A SPDY session already exists = and should be used instead of this connection.
    SPDY_SESSION_ALREADY_EXISTS = 143,

    // Error -144 was removed (LIMIT_VIOLATION,.

    // Websocket protocol error. Indicates that we are terminating the connection
    // due to a malformed frame or other protocol violation.
    WS_PROTOCOL_ERROR = 145,

    // Error -146 was removed (PROTOCOL_SWITCHED,

    // Returned when attempting to bind an address that is already in use.
    ADDRESS_IN_USE = 147,

    // An operation failed because the SSL handshake has not completed.
    SSL_HANDSHAKE_NOT_COMPLETED = 148,

    // SSL peer's public key is invalid.
    SSL_BAD_PEER_PUBLIC_KEY = 149,

    // The certificate didn't match the built-in public key pins for the host name.
    // The pins are set in net/http/transport_security_state.cc and require that
    // one of a set of public keys exist on the path from the leaf to the root.
    SSL_PINNED_KEY_NOT_IN_CERT_CHAIN = 150,

    // Server request for client certificate did not contain any types we support.
    CLIENT_AUTH_CERT_TYPE_UNSUPPORTED = 151,

    // Server requested one type of cert = then requested a different type while the
    // first was still being generated.
    ORIGIN_BOUND_CERT_GENERATION_TYPE_MISMATCH = 152,

    // An SSL peer sent us a fatal decrypt_error alert. This typically occurs when
    // a peer could not correctly verify a signature (in CertificateVerify or
    // ServerKeyExchange, or validate a Finished message.
    SSL_DECRYPT_ERROR_ALERT = 153,

    // There are too many pending WebSocketJob instances = so the new job was not
    // pushed to the queue.
    WS_THROTTLE_QUEUE_TOO_LARGE = 154,

    // Error -155 was removed (TOO_MANY_SOCKET_STREAMS,

    // The SSL server certificate changed in a renegotiation.
    SSL_SERVER_CERT_CHANGED = 156,

    // The SSL server indicated that an unnecessary TLS version fallback was
    // performed.
    SSL_INAPPROPRIATE_FALLBACK = 157,

    // Certificate Transparency: All Signed Certificate Timestamps failed to verify.
    CT_NO_SCTS_VERIFIED_OK = 158,

    // The SSL server sent us a fatal unrecognized_name alert.
    SSL_UNRECOGNIZED_NAME_ALERT = 159,

    // Failed to set the socket's receive buffer size as requested.
    SOCKET_SET_RECEIVE_BUFFER_SIZE_ERROR = 160,

    // Failed to set the socket's send buffer size as requested.
    SOCKET_SET_SEND_BUFFER_SIZE_ERROR = 161,

    // Failed to set the socket's receive buffer size as requested = despite success
    // return code from setsockopt.
    SOCKET_RECEIVE_BUFFER_SIZE_UNCHANGEABLE = 162,

    // Failed to set the socket's send buffer size as requested = despite success
    // return code from setsockopt.
    SOCKET_SEND_BUFFER_SIZE_UNCHANGEABLE = 163,

    // Failed to import a client certificate from the platform store into the SSL
    // library.
    SSL_CLIENT_AUTH_CERT_BAD_FORMAT = 164,

    // The SSL server requires falling back to a version older than the configured
    // minimum fallback version = and thus fallback failed.
    SSL_FALLBACK_BEYOND_MINIMUM_VERSION = 165,

    // Certificate error codes
    //
    // The values of certificate error codes must be consecutive.

    // The server responded with a certificate whose common name did not match
    // the host name.  This could mean:
    //
    // 1. An attacker has redirected our traffic to his server and is
    //    presenting a certificate for which he knows the private key.
    //
    // 2. The server is misconfigured and responding with the wrong cert.
    //
    // 3. The user is on a wireless network and is being redirected to the
    //    network's login page.
    //
    // 4. The OS has used a DNS search suffix and the server doesn't have
    //    a certificate for the abbreviated name in the address bar.
    //
    CERT_COMMON_NAME_INVALID = 200,

    // The server responded with a certificate that = by our clock = appears to
    // either not yet be valid or to have expired.  This could mean:
    //
    // 1. An attacker is presenting an old certificate for which he has
    //    managed to obtain the private key.
    //
    // 2. The server is misconfigured and is not presenting a valid cert.
    //
    // 3. Our clock is wrong.
    //
    CERT_DATE_INVALID = 201,

    // The server responded with a certificate that is signed by an authority
    // we don't trust.  The could mean:
    //
    // 1. An attacker has substituted the real certificate for a cert that
    //    contains his public key and is signed by his cousin.
    //
    // 2. The server operator has a legitimate certificate from a CA we don't
    //    know about = but should trust.
    //
    // 3. The server is presenting a self-signed certificate = providing no
    //    defense against active attackers (but foiling passive attackers,.
    //
    CERT_AUTHORITY_INVALID = 202,

    // The server responded with a certificate that contains errors.
    // This error is not recoverable.
    //
    // MSDN describes this error as follows:
    //   "The SSL certificate contains errors."
    // NOTE: It's unclear how this differs from ERR_CERT_INVALID. For consistency,
    // use that code instead of this one from now on.
    //
    CERT_CONTAINS_ERRORS = 203,

    // The certificate has no mechanism for determining if it is revoked.  In
    // effect = this certificate cannot be revoked.
    CERT_NO_REVOCATION_MECHANISM = 204,

    // Revocation information for the security certificate for this site is not
    // available.  This could mean:
    //
    // 1. An attacker has compromised the private key in the certificate and is
    //    blocking our attempt to find out that the cert was revoked.
    //
    // 2. The certificate is unrevoked = but the revocation server is busy or
    //    unavailable.
    //
    CERT_UNABLE_TO_CHECK_REVOCATION = 205,

    // The server responded with a certificate has been revoked.
    // We have the capability to ignore this error = but it is probably not the
    // thing to do.
    CERT_REVOKED = 206,

    // The server responded with a certificate that is invalid.
    // This error is not recoverable.
    //
    // MSDN describes this error as follows:
    //   "The SSL certificate is invalid."
    //
    CERT_INVALID = 207,

    // The server responded with a certificate that is signed using a weak
    // signature algorithm.
    CERT_WEAK_SIGNATURE_ALGORITHM = 208,

    // -209 is availible: was CERT_NOT_IN_DNS.

    // The host name specified in the certificate is not unique.
    CERT_NON_UNIQUE_NAME = 210,

    // The server responded with a certificate that contains a weak key (e.g.
    // a too-small RSA key,.
    CERT_WEAK_KEY = 211,

    // The certificate claimed DNS names that are in violation of name constraints.
    CERT_NAME_CONSTRAINT_VIOLATION = 212,

    // The certificate's validity period is too long.
    CERT_VALIDITY_TOO_LONG = 213,

    // Add new certificate error codes here.
    //
    // Update the value of CERT_END whenever you add a new certificate error
    // code.

    // The value immediately past the last certificate error code.
    CERT_END = 214,

    // The URL is invalid.
    INVALID_URL = 300,

    // The scheme of the URL is disallowed.
    DISALLOWED_URL_SCHEME = 301,

    // The scheme of the URL is unknown.
    UNKNOWN_URL_SCHEME = 302,

    // Attempting to load an URL resulted in too many redirects.
    TOO_MANY_REDIRECTS = 310,

    // Attempting to load an URL resulted in an unsafe redirect (e.g. = a redirect
    // to file:// is considered unsafe,.
    UNSAFE_REDIRECT = 311,

    // Attempting to load an URL with an unsafe port number.  These are port
    // numbers that correspond to services = which are not robust to spurious input
    // that may be constructed as a result of an allowed web construct (e.g. = HTTP
    // looks a lot like SMTP = so form submission to port 25 is denied,.
    UNSAFE_PORT = 312,

    // The server's response was invalid.
    INVALID_RESPONSE = 320,

    // Error in chunked transfer encoding.
    INVALID_CHUNKED_ENCODING = 321,

    // The server did not support the request method.
    METHOD_NOT_SUPPORTED = 322,

    // The response was 407 (Proxy Authentication Required, = yet we did not send
    // the request to a proxy.
    UNEXPECTED_PROXY_AUTH = 323,

    // The server closed the connection without sending any data.
    EMPTY_RESPONSE = 324,

    // The headers section of the response is too large.
    RESPONSE_HEADERS_TOO_BIG = 325,

    // The PAC requested by HTTP did not have a valid status code (non-200,.
    PAC_STATUS_NOT_OK = 326,

    // The evaluation of the PAC script failed.
    PAC_SCRIPT_FAILED = 327,

    // The response was 416 (Requested range not satisfiable, and the server cannot
    // satisfy the range requested.
    REQUEST_RANGE_NOT_SATISFIABLE = 328,

    // The identity used for authentication is invalid.
    MALFORMED_IDENTITY = 329,

    // Content decoding of the response body failed.
    CONTENT_DECODING_FAILED = 330,

    // An operation could not be completed because all network IO
    // is suspended.
    NETWORK_IO_SUSPENDED = 331,

    // FLIP data received without receiving a SYN_REPLY on the stream.
    SYN_REPLY_NOT_RECEIVED = 332,

    // Converting the response to target encoding failed.
    ENCODING_CONVERSION_FAILED = 333,

    // The server sent an FTP directory listing in a format we do not understand.
    UNRECOGNIZED_FTP_DIRECTORY_LISTING_FORMAT = 334,

    // Attempted use of an unknown SPDY stream id.
    INVALID_SPDY_STREAM = 335,

    // There are no supported proxies in the provided list.
    NO_SUPPORTED_PROXIES = 336,

    // There is a SPDY protocol error.
    SPDY_PROTOCOL_ERROR = 337,

    // Credentials could not be established during HTTP Authentication.
    INVALID_AUTH_CREDENTIALS = 338,

    // An HTTP Authentication scheme was tried which is not supported on this
    // machine.
    UNSUPPORTED_AUTH_SCHEME = 339,

    // Detecting the encoding of the response failed.
    ENCODING_DETECTION_FAILED = 340,

    // (GSSAPI, No Kerberos credentials were available during HTTP Authentication.
    MISSING_AUTH_CREDENTIALS = 341,

    // An unexpected = but documented = SSPI or GSSAPI status code was returned.
    UNEXPECTED_SECURITY_LIBRARY_STATUS = 342,

    // The environment was not set up correctly for authentication (for
    // example = no KDC could be found or the principal is unknown.
    MISCONFIGURED_AUTH_ENVIRONMENT = 343,

    // An undocumented SSPI or GSSAPI status code was returned.
    UNDOCUMENTED_SECURITY_LIBRARY_STATUS = 344,

    // The HTTP response was too big to drain.
    RESPONSE_BODY_TOO_BIG_TO_DRAIN = 345,

    // The HTTP response contained multiple distinct Content-Length headers.
    RESPONSE_HEADERS_MULTIPLE_CONTENT_LENGTH = 346,

    // SPDY Headers have been received = but not all of them - status or version
    // headers are missing = so we're expecting additional frames to complete them.
    INCOMPLETE_SPDY_HEADERS = 347,

    // No PAC URL configuration could be retrieved from DHCP. This can indicate
    // either a failure to retrieve the DHCP configuration = or that there was no
    // PAC URL configured in DHCP.
    PAC_NOT_IN_DHCP = 348,

    // The HTTP response contained multiple Content-Disposition headers.
    RESPONSE_HEADERS_MULTIPLE_CONTENT_DISPOSITION = 349,

    // The HTTP response contained multiple Location headers.
    RESPONSE_HEADERS_MULTIPLE_LOCATION = 350,

    // SPDY server refused the stream. Client should retry. This should never be a
    // user-visible error.
    SPDY_SERVER_REFUSED_STREAM = 351,

    // SPDY server didn't respond to the PING message.
    SPDY_PING_FAILED = 352,

    // Obsolete.  Kept here to avoid reuse = as the old error can still appear on
    // histograms.
    // PIPELINE_EVICTION = 353,

    // The HTTP response body transferred fewer bytes than were advertised by the
    // Content-Length header when the connection is closed.
    CONTENT_LENGTH_MISMATCH = 354,

    // The HTTP response body is transferred with Chunked-Encoding = but the
    // terminating zero-length chunk was never sent when the connection is closed.
    INCOMPLETE_CHUNKED_ENCODING = 355,

    // There is a QUIC protocol error.
    QUIC_PROTOCOL_ERROR = 356,

    // The HTTP headers were truncated by an EOF.
    RESPONSE_HEADERS_TRUNCATED = 357,

    // The QUIC crytpo handshake failed.  This means that the server was unable
    // to read any requests sent = so they may be resent.
    QUIC_HANDSHAKE_FAILED = 358,

    // An https resource was requested over an insecure QUIC connection.
    REQUEST_FOR_SECURE_RESOURCE_OVER_INSECURE_QUIC = 359,

    // Transport security is inadequate for the SPDY version.
    SPDY_INADEQUATE_TRANSPORT_SECURITY = 360,

    // The peer violated SPDY flow control.
    SPDY_FLOW_CONTROL_ERROR = 361,

    // The peer sent an improperly sized SPDY frame.
    SPDY_FRAME_SIZE_ERROR = 362,

    // Decoding or encoding of compressed SPDY headers failed.
    SPDY_COMPRESSION_ERROR = 363,

    // Proxy Auth Requested without a valid Client Socket Handle.
    PROXY_AUTH_REQUESTED_WITH_NO_CONNECTION = 364,

    // HTTP_1_1_REQUIRED error code received on HTTP/2 session.
    HTTP_1_1_REQUIRED = 365,

    // HTTP_1_1_REQUIRED error code received on HTTP/2 session to proxy.
    PROXY_HTTP_1_1_REQUIRED = 366,

    // The cache does not have the requested entry.
    CACHE_MISS = 400,

    // Unable to read from the disk cache.
    CACHE_READ_FAILURE = 401,

    // Unable to write to the disk cache.
    CACHE_WRITE_FAILURE = 402,

    // The operation is not supported for this entry.
    CACHE_OPERATION_NOT_SUPPORTED = 403,

    // The disk cache is unable to open this entry.
    CACHE_OPEN_FAILURE = 404,

    // The disk cache is unable to create this entry.
    CACHE_CREATE_FAILURE = 405,

    // Multiple transactions are racing to create disk cache entries. This is an
    // internal error returned from the HttpCache to the HttpCacheTransaction that
    // tells the transaction to restart the entry-creation logic because the state
    // of the cache has changed.
    CACHE_RACE = 406,

    // The cache was unable to read a checksum record on an entry. This can be
    // returned from attempts to read from the cache. It is an internal error,
    // returned by the SimpleCache backend = but not by any URLRequest methods
    // or members.
    CACHE_CHECKSUM_READ_FAILURE = 407,

    // The cache found an entry with an invalid checksum. This can be returned from
    // attempts to read from the cache. It is an internal error = returned by the
    // SimpleCache backend = but not by any URLRequest methods or members.
    CACHE_CHECKSUM_MISMATCH = 408,

    // Internal error code for the HTTP cache. The cache lock timeout has fired.
    CACHE_LOCK_TIMEOUT = 409,

    // The server's response was insecure (e.g. there was a cert error,.
    INSECURE_RESPONSE = 501,

    // The server responded to a <keygen> with a generated client cert that we
    // don't have the matching private key for.
    NO_PRIVATE_KEY_FOR_CERT = 502,

    // An error adding to the OS certificate database (e.g. OS X Keychain,.
    ADD_USER_CERT_FAILED = 503,

    // *** Code -600 is reserved (was FTP_PASV_COMMAND_FAILED,. ***

    // A generic error for failed FTP control connection command.
    // If possible = please use or add a more specific error code.
    FTP_FAILED = 601,

    // The server cannot fulfill the request at this point. This is a temporary
    // error.
    // FTP response code 421.
    FTP_SERVICE_UNAVAILABLE = 602,

    // The server has aborted the transfer.
    // FTP response code 426.
    FTP_TRANSFER_ABORTED = 603,

    // The file is busy = or some other temporary error condition on opening
    // the file.
    // FTP response code 450.
    FTP_FILE_BUSY = 604,

    // Server rejected our command because of syntax errors.
    // FTP response codes 500 = 501.
    FTP_SYNTAX_ERROR = 605,

    // Server does not support the command we issued.
    // FTP response codes 502 = 504.
    FTP_COMMAND_NOT_SUPPORTED = 606,

    // Server rejected our command because we didn't issue the commands in right
    // order.
    // FTP response code 503.
    FTP_BAD_COMMAND_SEQUENCE = 607,

    // PKCS #12 import failed due to incorrect password.
    PKCS12_IMPORT_BAD_PASSWORD = 701,

    // PKCS #12 import failed due to other error.
    PKCS12_IMPORT_FAILED = 702,

    // CA import failed - not a CA cert.
    IMPORT_CA_CERT_NOT_CA = 703,

    // Import failed - certificate already exists in database.
    // Note it's a little weird this is an error but reimporting a PKCS12 is ok
    // (no-op,.  That's how Mozilla does it = though.
    IMPORT_CERT_ALREADY_EXISTS = 704,

    // CA import failed due to some other error.
    IMPORT_CA_CERT_FAILED = 705,

    // Server certificate import failed due to some internal error.
    IMPORT_SERVER_CERT_FAILED = 706,

    // PKCS #12 import failed due to invalid MAC.
    PKCS12_IMPORT_INVALID_MAC = 707,

    // PKCS #12 import failed due to invalid/corrupt file.
    PKCS12_IMPORT_INVALID_FILE = 708,

    // PKCS #12 import failed due to unsupported features.
    PKCS12_IMPORT_UNSUPPORTED = 709,

    // Key generation failed.
    KEY_GENERATION_FAILED = 710,

    // Server-bound certificate generation failed.
    ORIGIN_BOUND_CERT_GENERATION_FAILED = 711,

    // Failure to export private key.
    PRIVATE_KEY_EXPORT_FAILED = 712,

    // Self-signed certificate generation failed.
    SELF_SIGNED_CERT_GENERATION_FAILED = 713,

    // The certificate database changed in some way.
    CERT_DATABASE_CHANGED = 714,

    // Failure to import Channel ID.
    CHANNEL_ID_IMPORT_FAILED = 715,

    // DNS error codes.

    // DNS resolver received a malformed response.
    DNS_MALFORMED_RESPONSE = 800,

    // DNS server requires TCP
    DNS_SERVER_REQUIRES_TCP = 801,

    // DNS server failed.  This error is returned for all of the following
    // error conditions:
    // 1 - Format error - The name server was unable to interpret the query.
    // 2 - Server failure - The name server was unable to process this query
    //     due to a problem with the name server.
    // 4 - Not Implemented - The name server does not support the requested
    //     kind of query.
    // 5 - Refused - The name server refuses to perform the specified
    //     operation for policy reasons.
    DNS_SERVER_FAILED = 802,

    // DNS transaction timed out.
    DNS_TIMED_OUT = 803,

    // The entry was not found in cache = for cache-only lookups.
    DNS_CACHE_MISS = 804,

    // Suffix search list rules prevent resolution of the given host name.
    DNS_SEARCH_EMPTY = 805,

    // Failed to sort addresses according to RFC3484.
    DNS_SORT_ERROR = 806,
}
