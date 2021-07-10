import binascii
from unittest import TestCase, skipIf

from aioquic.buffer import Buffer
from aioquic.quic.crypto import (
    INITIAL_CIPHER_SUITE,
    CryptoError,
    CryptoPair,
    derive_key_iv_hp,
)
from aioquic.quic.packet import PACKET_FIXED_BIT, QuicProtocolVersion
from aioquic.tls import CipherSuite

from .utils import SKIP_TESTS

PROTOCOL_VERSION = QuicProtocolVersion.DRAFT_25

CHACHA20_CLIENT_PACKET_NUMBER = 2
CHACHA20_CLIENT_PLAIN_HEADER = binascii.unhexlify(
    "e1ff0000160880b57c7b70d8524b0850fc2a28e240fd7640170002"
)
CHACHA20_CLIENT_PLAIN_PAYLOAD = binascii.unhexlify("0201000000")
CHACHA20_CLIENT_ENCRYPTED_PACKET = binascii.unhexlify(
    "e8ff0000160880b57c7b70d8524b0850fc2a28e240fd7640178313b04be98449"
    "eb10567e25ce930381f2a5b7da2db8db"
)

LONG_CLIENT_PACKET_NUMBER = 2
LONG_CLIENT_PLAIN_HEADER = binascii.unhexlify(
    "c3ff000017088394c8f03e5157080000449e00000002"
)
LONG_CLIENT_PLAIN_PAYLOAD = binascii.unhexlify(
    "060040c4010000c003036660261ff947cea49cce6cfad687f457cf1b14531ba1"
    "4131a0e8f309a1d0b9c4000006130113031302010000910000000b0009000006"
    "736572766572ff01000100000a00140012001d00170018001901000101010201"
    "03010400230000003300260024001d00204cfdfcd178b784bf328cae793b136f"
    "2aedce005ff183d7bb1495207236647037002b0003020304000d0020001e0403"
    "05030603020308040805080604010501060102010402050206020202002d0002"
    "0101001c00024001"
) + bytes(962)
LONG_CLIENT_ENCRYPTED_PACKET = binascii.unhexlify(
    "c0ff000017088394c8f03e5157080000449e3b343aa8535064a4268a0d9d7b1c"
    "9d250ae355162276e9b1e3011ef6bbc0ab48ad5bcc2681e953857ca62becd752"
    "4daac473e68d7405fbba4e9ee616c87038bdbe908c06d9605d9ac49030359eec"
    "b1d05a14e117db8cede2bb09d0dbbfee271cb374d8f10abec82d0f59a1dee29f"
    "e95638ed8dd41da07487468791b719c55c46968eb3b54680037102a28e53dc1d"
    "12903db0af5821794b41c4a93357fa59ce69cfe7f6bdfa629eef78616447e1d6"
    "11c4baf71bf33febcb03137c2c75d25317d3e13b684370f668411c0f00304b50"
    "1c8fd422bd9b9ad81d643b20da89ca0525d24d2b142041cae0af205092e43008"
    "0cd8559ea4c5c6e4fa3f66082b7d303e52ce0162baa958532b0bbc2bc785681f"
    "cf37485dff6595e01e739c8ac9efba31b985d5f656cc092432d781db95221724"
    "87641c4d3ab8ece01e39bc85b15436614775a98ba8fa12d46f9b35e2a55eb72d"
    "7f85181a366663387ddc20551807e007673bd7e26bf9b29b5ab10a1ca87cbb7a"
    "d97e99eb66959c2a9bc3cbde4707ff7720b110fa95354674e395812e47a0ae53"
    "b464dcb2d1f345df360dc227270c750676f6724eb479f0d2fbb6124429990457"
    "ac6c9167f40aab739998f38b9eccb24fd47c8410131bf65a52af841275d5b3d1"
    "880b197df2b5dea3e6de56ebce3ffb6e9277a82082f8d9677a6767089b671ebd"
    "244c214f0bde95c2beb02cd1172d58bdf39dce56ff68eb35ab39b49b4eac7c81"
    "5ea60451d6e6ab82119118df02a586844a9ffe162ba006d0669ef57668cab38b"
    "62f71a2523a084852cd1d079b3658dc2f3e87949b550bab3e177cfc49ed190df"
    "f0630e43077c30de8f6ae081537f1e83da537da980afa668e7b7fb25301cf741"
    "524be3c49884b42821f17552fbd1931a813017b6b6590a41ea18b6ba49cd48a4"
    "40bd9a3346a7623fb4ba34a3ee571e3c731f35a7a3cf25b551a680fa68763507"
    "b7fde3aaf023c50b9d22da6876ba337eb5e9dd9ec3daf970242b6c5aab3aa4b2"
    "96ad8b9f6832f686ef70fa938b31b4e5ddd7364442d3ea72e73d668fb0937796"
    "f462923a81a47e1cee7426ff6d9221269b5a62ec03d6ec94d12606cb485560ba"
    "b574816009e96504249385bb61a819be04f62c2066214d8360a2022beb316240"
    "b6c7d78bbe56c13082e0ca272661210abf020bf3b5783f1426436cf9ff418405"
    "93a5d0638d32fc51c5c65ff291a3a7a52fd6775e623a4439cc08dd25582febc9"
    "44ef92d8dbd329c91de3e9c9582e41f17f3d186f104ad3f90995116c682a2a14"
    "a3b4b1f547c335f0be710fc9fc03e0e587b8cda31ce65b969878a4ad4283e6d5"
    "b0373f43da86e9e0ffe1ae0fddd3516255bd74566f36a38703d5f34249ded1f6"
    "6b3d9b45b9af2ccfefe984e13376b1b2c6404aa48c8026132343da3f3a33659e"
    "c1b3e95080540b28b7f3fcd35fa5d843b579a84c089121a60d8c1754915c344e"
    "eaf45a9bf27dc0c1e78416169122091313eb0e87555abd706626e557fc36a04f"
    "cd191a58829104d6075c5594f627ca506bf181daec940f4a4f3af0074eee89da"
    "acde6758312622d4fa675b39f728e062d2bee680d8f41a597c262648bb18bcfc"
    "13c8b3d97b1a77b2ac3af745d61a34cc4709865bac824a94bb19058015e4e42d"
    "c9be6c7803567321829dd85853396269"
)

LONG_SERVER_PACKET_NUMBER = 1
LONG_SERVER_PLAIN_HEADER = binascii.unhexlify(
    "c1ff0000170008f067a5502a4262b50040740001"
)
LONG_SERVER_PLAIN_PAYLOAD = binascii.unhexlify(
    "0d0000000018410a020000560303eefce7f7b37ba1d1632e96677825ddf73988"
    "cfc79825df566dc5430b9a045a1200130100002e00330024001d00209d3c940d"
    "89690b84d08a60993c144eca684d1081287c834d5311bcf32bb9da1a002b0002"
    "0304"
)
LONG_SERVER_ENCRYPTED_PACKET = binascii.unhexlify(
    "c9ff0000170008f067a5502a4262b5004074168bf22b7002596f99ae67abf65a"
    "5852f54f58c37c808682e2e40492d8a3899fb04fc0afe9aabc8767b18a0aa493"
    "537426373b48d502214dd856d63b78cee37bc664b3fe86d487ac7a77c53038a3"
    "cd32f0b5004d9f5754c4f7f2d1f35cf3f7116351c92b9cf9bb6d091ddfc8b32d"
    "432348a2c413"
)

SHORT_SERVER_PACKET_NUMBER = 3
SHORT_SERVER_PLAIN_HEADER = binascii.unhexlify("41b01fd24a586a9cf30003")
SHORT_SERVER_PLAIN_PAYLOAD = binascii.unhexlify(
    "06003904000035000151805a4bebf5000020b098c8dc4183e4c182572e10ac3e"
    "2b88897e0524c8461847548bd2dffa2c0ae60008002a0004ffffffff"
)
SHORT_SERVER_ENCRYPTED_PACKET = binascii.unhexlify(
    "5db01fd24a586a9cf33dec094aaec6d6b4b7a5e15f5a3f05d06cf1ad0355c19d"
    "cce0807eecf7bf1c844a66e1ecd1f74b2a2d69bfd25d217833edd973246597bd"
    "5107ea15cb1e210045396afa602fe23432f4ab24ce251b"
)


class CryptoTest(TestCase):
    """
    Test vectors from:

    https://tools.ietf.org/html/draft-ietf-quic-tls-18#appendix-A
    """

    def create_crypto(self, is_client):
        pair = CryptoPair()
        pair.setup_initial(
            cid=binascii.unhexlify("8394c8f03e515708"),
            is_client=is_client,
            version=PROTOCOL_VERSION,
        )
        return pair

    def test_derive_key_iv_hp(self):
        # client
        secret = binascii.unhexlify(
            "8a3515a14ae3c31b9c2d6d5bc58538ca5cd2baa119087143e60887428dcb52f6"
        )
        key, iv, hp = derive_key_iv_hp(INITIAL_CIPHER_SUITE, secret)
        self.assertEqual(key, binascii.unhexlify("98b0d7e5e7a402c67c33f350fa65ea54"))
        self.assertEqual(iv, binascii.unhexlify("19e94387805eb0b46c03a788"))
        self.assertEqual(hp, binascii.unhexlify("0edd982a6ac527f2eddcbb7348dea5d7"))

        # server
        secret = binascii.unhexlify(
            "47b2eaea6c266e32c0697a9e2a898bdf5c4fb3e5ac34f0e549bf2c58581a3811"
        )
        key, iv, hp = derive_key_iv_hp(INITIAL_CIPHER_SUITE, secret)
        self.assertEqual(key, binascii.unhexlify("9a8be902a9bdd91d16064ca118045fb4"))
        self.assertEqual(iv, binascii.unhexlify("0a82086d32205ba22241d8dc"))
        self.assertEqual(hp, binascii.unhexlify("94b9452d2b3c7c7f6da7fdd8593537fd"))

    @skipIf("chacha20" in SKIP_TESTS, "Skipping chacha20 tests")
    def test_decrypt_chacha20(self):
        pair = CryptoPair()
        pair.recv.setup(
            cipher_suite=CipherSuite.CHACHA20_POLY1305_SHA256,
            secret=binascii.unhexlify(
                "b42772df33c9719a32820d302aa664d080d7f5ea7a71a330f87864cb289ae8c0"
            ),
            version=PROTOCOL_VERSION,
        )

        plain_header, plain_payload, packet_number = pair.decrypt_packet(
            CHACHA20_CLIENT_ENCRYPTED_PACKET, 25, 0
        )
        self.assertEqual(plain_header, CHACHA20_CLIENT_PLAIN_HEADER)
        self.assertEqual(plain_payload, CHACHA20_CLIENT_PLAIN_PAYLOAD)
        self.assertEqual(packet_number, CHACHA20_CLIENT_PACKET_NUMBER)

    def test_decrypt_long_client(self):
        pair = self.create_crypto(is_client=False)

        plain_header, plain_payload, packet_number = pair.decrypt_packet(
            LONG_CLIENT_ENCRYPTED_PACKET, 18, 0
        )
        self.assertEqual(plain_header, LONG_CLIENT_PLAIN_HEADER)
        self.assertEqual(plain_payload, LONG_CLIENT_PLAIN_PAYLOAD)
        self.assertEqual(packet_number, LONG_CLIENT_PACKET_NUMBER)

    def test_decrypt_long_server(self):
        pair = self.create_crypto(is_client=True)

        plain_header, plain_payload, packet_number = pair.decrypt_packet(
            LONG_SERVER_ENCRYPTED_PACKET, 18, 0
        )
        self.assertEqual(plain_header, LONG_SERVER_PLAIN_HEADER)
        self.assertEqual(plain_payload, LONG_SERVER_PLAIN_PAYLOAD)
        self.assertEqual(packet_number, LONG_SERVER_PACKET_NUMBER)

    def test_decrypt_no_key(self):
        pair = CryptoPair()
        with self.assertRaises(CryptoError):
            pair.decrypt_packet(LONG_SERVER_ENCRYPTED_PACKET, 18, 0)

    def test_decrypt_short_server(self):
        pair = CryptoPair()
        pair.recv.setup(
            cipher_suite=INITIAL_CIPHER_SUITE,
            secret=binascii.unhexlify(
                "310281977cb8c1c1c1212d784b2d29e5a6489e23de848d370a5a2f9537f3a100"
            ),
            version=PROTOCOL_VERSION,
        )

        plain_header, plain_payload, packet_number = pair.decrypt_packet(
            SHORT_SERVER_ENCRYPTED_PACKET, 9, 0
        )
        self.assertEqual(plain_header, SHORT_SERVER_PLAIN_HEADER)
        self.assertEqual(plain_payload, SHORT_SERVER_PLAIN_PAYLOAD)
        self.assertEqual(packet_number, SHORT_SERVER_PACKET_NUMBER)

    @skipIf("chacha20" in SKIP_TESTS, "Skipping chacha20 tests")
    def test_encrypt_chacha20(self):
        pair = CryptoPair()
        pair.send.setup(
            cipher_suite=CipherSuite.CHACHA20_POLY1305_SHA256,
            secret=binascii.unhexlify(
                "b42772df33c9719a32820d302aa664d080d7f5ea7a71a330f87864cb289ae8c0"
            ),
            version=PROTOCOL_VERSION,
        )

        packet = pair.encrypt_packet(
            CHACHA20_CLIENT_PLAIN_HEADER,
            CHACHA20_CLIENT_PLAIN_PAYLOAD,
            CHACHA20_CLIENT_PACKET_NUMBER,
        )
        self.assertEqual(packet, CHACHA20_CLIENT_ENCRYPTED_PACKET)

    def test_encrypt_long_client(self):
        pair = self.create_crypto(is_client=True)

        packet = pair.encrypt_packet(
            LONG_CLIENT_PLAIN_HEADER,
            LONG_CLIENT_PLAIN_PAYLOAD,
            LONG_CLIENT_PACKET_NUMBER,
        )
        self.assertEqual(packet, LONG_CLIENT_ENCRYPTED_PACKET)

    def test_encrypt_long_server(self):
        pair = self.create_crypto(is_client=False)

        packet = pair.encrypt_packet(
            LONG_SERVER_PLAIN_HEADER,
            LONG_SERVER_PLAIN_PAYLOAD,
            LONG_SERVER_PACKET_NUMBER,
        )
        self.assertEqual(packet, LONG_SERVER_ENCRYPTED_PACKET)

    def test_encrypt_short_server(self):
        pair = CryptoPair()
        pair.send.setup(
            cipher_suite=INITIAL_CIPHER_SUITE,
            secret=binascii.unhexlify(
                "310281977cb8c1c1c1212d784b2d29e5a6489e23de848d370a5a2f9537f3a100"
            ),
            version=PROTOCOL_VERSION,
        )

        packet = pair.encrypt_packet(
            SHORT_SERVER_PLAIN_HEADER,
            SHORT_SERVER_PLAIN_PAYLOAD,
            SHORT_SERVER_PACKET_NUMBER,
        )
        self.assertEqual(packet, SHORT_SERVER_ENCRYPTED_PACKET)

    def test_key_update(self):
        pair1 = self.create_crypto(is_client=True)
        pair2 = self.create_crypto(is_client=False)

        def create_packet(key_phase, packet_number):
            buf = Buffer(capacity=100)
            buf.push_uint8(PACKET_FIXED_BIT | key_phase << 2 | 1)
            buf.push_bytes(binascii.unhexlify("8394c8f03e515708"))
            buf.push_uint16(packet_number)
            return buf.data, b"\x00\x01\x02\x03"

        def send(sender, receiver, packet_number=0):
            plain_header, plain_payload = create_packet(
                key_phase=sender.key_phase, packet_number=packet_number
            )
            encrypted = sender.encrypt_packet(
                plain_header, plain_payload, packet_number
            )
            recov_header, recov_payload, recov_packet_number = receiver.decrypt_packet(
                encrypted, len(plain_header) - 2, 0
            )
            self.assertEqual(recov_header, plain_header)
            self.assertEqual(recov_payload, plain_payload)
            self.assertEqual(recov_packet_number, packet_number)

        # roundtrip
        send(pair1, pair2, 0)
        send(pair2, pair1, 0)
        self.assertEqual(pair1.key_phase, 0)
        self.assertEqual(pair2.key_phase, 0)

        # pair 1 key update
        pair1.update_key()

        # roundtrip
        send(pair1, pair2, 1)
        send(pair2, pair1, 1)
        self.assertEqual(pair1.key_phase, 1)
        self.assertEqual(pair2.key_phase, 1)

        # pair 2 key update
        pair2.update_key()

        # roundtrip
        send(pair2, pair1, 2)
        send(pair1, pair2, 2)
        self.assertEqual(pair1.key_phase, 0)
        self.assertEqual(pair2.key_phase, 0)

        # pair 1 key - update, but not next to send
        pair1.update_key()

        # roundtrip
        send(pair2, pair1, 3)
        send(pair1, pair2, 3)
        self.assertEqual(pair1.key_phase, 1)
        self.assertEqual(pair2.key_phase, 1)
