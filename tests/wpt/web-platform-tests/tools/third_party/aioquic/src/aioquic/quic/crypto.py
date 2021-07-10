import binascii
from typing import Optional, Tuple

from .._crypto import AEAD, CryptoError, HeaderProtection
from ..tls import CipherSuite, cipher_suite_hash, hkdf_expand_label, hkdf_extract
from .packet import decode_packet_number, is_long_header

CIPHER_SUITES = {
    CipherSuite.AES_128_GCM_SHA256: (b"aes-128-ecb", b"aes-128-gcm"),
    CipherSuite.AES_256_GCM_SHA384: (b"aes-256-ecb", b"aes-256-gcm"),
    CipherSuite.CHACHA20_POLY1305_SHA256: (b"chacha20", b"chacha20-poly1305"),
}
INITIAL_CIPHER_SUITE = CipherSuite.AES_128_GCM_SHA256
INITIAL_SALT = binascii.unhexlify("c3eef712c72ebb5a11a7d2432bb46365bef9f502")
SAMPLE_SIZE = 16


class KeyUnavailableError(CryptoError):
    pass


def derive_key_iv_hp(
    cipher_suite: CipherSuite, secret: bytes
) -> Tuple[bytes, bytes, bytes]:
    algorithm = cipher_suite_hash(cipher_suite)
    if cipher_suite in [
        CipherSuite.AES_256_GCM_SHA384,
        CipherSuite.CHACHA20_POLY1305_SHA256,
    ]:
        key_size = 32
    else:
        key_size = 16
    return (
        hkdf_expand_label(algorithm, secret, b"quic key", b"", key_size),
        hkdf_expand_label(algorithm, secret, b"quic iv", b"", 12),
        hkdf_expand_label(algorithm, secret, b"quic hp", b"", key_size),
    )


class CryptoContext:
    def __init__(self, key_phase: int = 0) -> None:
        self.aead: Optional[AEAD] = None
        self.cipher_suite: Optional[CipherSuite] = None
        self.hp: Optional[HeaderProtection] = None
        self.key_phase = key_phase
        self.secret: Optional[bytes] = None
        self.version: Optional[int] = None

    def decrypt_packet(
        self, packet: bytes, encrypted_offset: int, expected_packet_number: int
    ) -> Tuple[bytes, bytes, int, bool]:
        if self.aead is None:
            raise KeyUnavailableError("Decryption key is not available")

        # header protection
        plain_header, packet_number = self.hp.remove(packet, encrypted_offset)
        first_byte = plain_header[0]

        # packet number
        pn_length = (first_byte & 0x03) + 1
        packet_number = decode_packet_number(
            packet_number, pn_length * 8, expected_packet_number
        )

        # detect key phase change
        crypto = self
        if not is_long_header(first_byte):
            key_phase = (first_byte & 4) >> 2
            if key_phase != self.key_phase:
                crypto = next_key_phase(self)

        # payload protection
        payload = crypto.aead.decrypt(
            packet[len(plain_header) :], plain_header, packet_number
        )

        return plain_header, payload, packet_number, crypto != self

    def encrypt_packet(
        self, plain_header: bytes, plain_payload: bytes, packet_number: int
    ) -> bytes:
        assert self.is_valid(), "Encryption key is not available"

        # payload protection
        protected_payload = self.aead.encrypt(
            plain_payload, plain_header, packet_number
        )

        # header protection
        return self.hp.apply(plain_header, protected_payload)

    def is_valid(self) -> bool:
        return self.aead is not None

    def setup(self, cipher_suite: CipherSuite, secret: bytes, version: int) -> None:
        hp_cipher_name, aead_cipher_name = CIPHER_SUITES[cipher_suite]

        key, iv, hp = derive_key_iv_hp(cipher_suite, secret)
        self.aead = AEAD(aead_cipher_name, key, iv)
        self.cipher_suite = cipher_suite
        self.hp = HeaderProtection(hp_cipher_name, hp)
        self.secret = secret
        self.version = version

    def teardown(self) -> None:
        self.aead = None
        self.cipher_suite = None
        self.hp = None
        self.secret = None


def apply_key_phase(self: CryptoContext, crypto: CryptoContext) -> None:
    self.aead = crypto.aead
    self.key_phase = crypto.key_phase
    self.secret = crypto.secret


def next_key_phase(self: CryptoContext) -> CryptoContext:
    algorithm = cipher_suite_hash(self.cipher_suite)

    crypto = CryptoContext(key_phase=int(not self.key_phase))
    crypto.setup(
        cipher_suite=self.cipher_suite,
        secret=hkdf_expand_label(
            algorithm, self.secret, b"quic ku", b"", algorithm.digest_size
        ),
        version=self.version,
    )
    return crypto


class CryptoPair:
    def __init__(self) -> None:
        self.aead_tag_size = 16
        self.recv = CryptoContext()
        self.send = CryptoContext()
        self._update_key_requested = False

    def decrypt_packet(
        self, packet: bytes, encrypted_offset: int, expected_packet_number: int
    ) -> Tuple[bytes, bytes, int]:
        plain_header, payload, packet_number, update_key = self.recv.decrypt_packet(
            packet, encrypted_offset, expected_packet_number
        )
        if update_key:
            self._update_key()
        return plain_header, payload, packet_number

    def encrypt_packet(
        self, plain_header: bytes, plain_payload: bytes, packet_number: int
    ) -> bytes:
        if self._update_key_requested:
            self._update_key()
        return self.send.encrypt_packet(plain_header, plain_payload, packet_number)

    def setup_initial(self, cid: bytes, is_client: bool, version: int) -> None:
        if is_client:
            recv_label, send_label = b"server in", b"client in"
        else:
            recv_label, send_label = b"client in", b"server in"

        algorithm = cipher_suite_hash(INITIAL_CIPHER_SUITE)
        initial_secret = hkdf_extract(algorithm, INITIAL_SALT, cid)
        self.recv.setup(
            cipher_suite=INITIAL_CIPHER_SUITE,
            secret=hkdf_expand_label(
                algorithm, initial_secret, recv_label, b"", algorithm.digest_size
            ),
            version=version,
        )
        self.send.setup(
            cipher_suite=INITIAL_CIPHER_SUITE,
            secret=hkdf_expand_label(
                algorithm, initial_secret, send_label, b"", algorithm.digest_size
            ),
            version=version,
        )

    def teardown(self) -> None:
        self.recv.teardown()
        self.send.teardown()

    def update_key(self) -> None:
        self._update_key_requested = True

    @property
    def key_phase(self) -> int:
        if self._update_key_requested:
            return int(not self.recv.key_phase)
        else:
            return self.recv.key_phase

    def _update_key(self) -> None:
        apply_key_phase(self.recv, next_key_phase(self.recv))
        apply_key_phase(self.send, next_key_phase(self.send))
        self._update_key_requested = False
