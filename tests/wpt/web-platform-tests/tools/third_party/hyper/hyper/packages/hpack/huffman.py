# -*- coding: utf-8 -*-
"""
hpack/huffman_decoder
~~~~~~~~~~~~~~~~~~~~~

An implementation of a bitwise prefix tree specially built for decoding
Huffman-coded content where we already know the Huffman table.
"""
from .compat import to_byte, decode_hex
from .exceptions import HPACKDecodingError

def _pad_binary(bin_str, req_len=8):
    """
    Given a binary string (returned by bin()), pad it to a full byte length.
    """
    bin_str = bin_str[2:]  # Strip the 0b prefix
    return max(0, req_len - len(bin_str)) * '0' + bin_str

def _hex_to_bin_str(hex_string):
    """
    Given a Python bytestring, returns a string representing those bytes in
    unicode form.
    """
    unpadded_bin_string_list = (bin(to_byte(c)) for c in hex_string)
    padded_bin_string_list = map(_pad_binary, unpadded_bin_string_list)
    bitwise_message = "".join(padded_bin_string_list)
    return bitwise_message


class HuffmanDecoder(object):
    """
    Decodes a Huffman-coded bytestream according to the Huffman table laid out
    in the HPACK specification.
    """
    class _Node(object):
        def __init__(self, data):
            self.data = data
            self.mapping = {}

    def __init__(self, huffman_code_list, huffman_code_list_lengths):
        self.root = self._Node(None)
        for index, (huffman_code, code_length) in enumerate(zip(huffman_code_list, huffman_code_list_lengths)):
            self._insert(huffman_code, code_length, index)

    def _insert(self, hex_number, hex_length, letter):
        """
        Inserts a Huffman code point into the tree.
        """
        hex_number = _pad_binary(bin(hex_number), hex_length)
        cur_node = self.root
        for digit in hex_number:
            if digit not in cur_node.mapping:
                cur_node.mapping[digit] = self._Node(None)
            cur_node = cur_node.mapping[digit]
        cur_node.data = letter

    def decode(self, encoded_string):
        """
        Decode the given Huffman coded string.
        """
        number = _hex_to_bin_str(encoded_string)
        cur_node = self.root
        decoded_message = bytearray()

        try:
            for digit in number:
                cur_node = cur_node.mapping[digit]
                if cur_node.data is not None:
                    # If we get EOS, everything else is padding.
                    if cur_node.data == 256:
                        break

                    decoded_message.append(cur_node.data)
                    cur_node = self.root
        except KeyError:
            # We have a Huffman-coded string that doesn't match our trie. This
            # is pretty bad: raise a useful exception.
            raise HPACKDecodingError("Invalid Huffman-coded string received.")
        return bytes(decoded_message)


class HuffmanEncoder(object):
    """
    Encodes a string according to the Huffman encoding table defined in the
    HPACK specification.
    """
    def __init__(self, huffman_code_list, huffman_code_list_lengths):
        self.huffman_code_list = huffman_code_list
        self.huffman_code_list_lengths = huffman_code_list_lengths

    def encode(self, bytes_to_encode):
        """
        Given a string of bytes, encodes them according to the HPACK Huffman
        specification.
        """
        # If handed the empty string, just immediately return.
        if not bytes_to_encode:
            return b''

        final_num = 0
        final_int_len = 0

        # Turn each byte into its huffman code. These codes aren't necessarily
        # octet aligned, so keep track of how far through an octet we are. To
        # handle this cleanly, just use a single giant integer.
        for char in bytes_to_encode:
            byte = to_byte(char)
            bin_int_len = self.huffman_code_list_lengths[byte]
            bin_int = self.huffman_code_list[byte] & (2 ** (bin_int_len + 1) - 1)
            final_num <<= bin_int_len
            final_num |= bin_int
            final_int_len += bin_int_len

        # Pad out to an octet with ones.
        bits_to_be_padded = (8 - (final_int_len % 8)) % 8
        final_num <<= bits_to_be_padded
        final_num |= (1 << (bits_to_be_padded)) - 1

        # Convert the number to hex and strip off the leading '0x' and the
        # trailing 'L', if present.
        final_num = hex(final_num)[2:].rstrip('L')

        # If this is odd, prepend a zero.
        final_num = '0' + final_num if len(final_num) % 2 != 0 else final_num

        # This number should have twice as many digits as bytes. If not, we're
        # missing some leading zeroes. Work out how many bytes we want and how
        # many digits we have, then add the missing zero digits to the front.
        total_bytes = (final_int_len + bits_to_be_padded) // 8
        expected_digits = total_bytes * 2

        if len(final_num) != expected_digits:
            missing_digits = expected_digits - len(final_num)
            final_num = ('0' * missing_digits) + final_num

        return decode_hex(final_num)
