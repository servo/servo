from unittest import TestCase

from aioquic.quic.events import StreamDataReceived
from aioquic.quic.packet import QuicStreamFrame
from aioquic.quic.packet_builder import QuicDeliveryState
from aioquic.quic.stream import QuicStream


class QuicStreamTest(TestCase):
    def test_recv_empty(self):
        stream = QuicStream(stream_id=0)
        self.assertEqual(bytes(stream._recv_buffer), b"")
        self.assertEqual(list(stream._recv_ranges), [])
        self.assertEqual(stream._recv_buffer_start, 0)

        # empty
        self.assertEqual(stream.add_frame(QuicStreamFrame(offset=0, data=b"")), None)
        self.assertEqual(bytes(stream._recv_buffer), b"")
        self.assertEqual(list(stream._recv_ranges), [])
        self.assertEqual(stream._recv_buffer_start, 0)

    def test_recv_ordered(self):
        stream = QuicStream(stream_id=0)

        # add data at start
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=0, data=b"01234567")),
            StreamDataReceived(data=b"01234567", end_stream=False, stream_id=0),
        )
        self.assertEqual(bytes(stream._recv_buffer), b"")
        self.assertEqual(list(stream._recv_ranges), [])
        self.assertEqual(stream._recv_buffer_start, 8)

        # add more data
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=8, data=b"89012345")),
            StreamDataReceived(data=b"89012345", end_stream=False, stream_id=0),
        )
        self.assertEqual(bytes(stream._recv_buffer), b"")
        self.assertEqual(list(stream._recv_ranges), [])
        self.assertEqual(stream._recv_buffer_start, 16)

        # add data and fin
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=16, data=b"67890123", fin=True)),
            StreamDataReceived(data=b"67890123", end_stream=True, stream_id=0),
        )
        self.assertEqual(bytes(stream._recv_buffer), b"")
        self.assertEqual(list(stream._recv_ranges), [])
        self.assertEqual(stream._recv_buffer_start, 24)

    def test_recv_unordered(self):
        stream = QuicStream(stream_id=0)

        # add data at offset 8
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=8, data=b"89012345")), None
        )
        self.assertEqual(
            bytes(stream._recv_buffer), b"\x00\x00\x00\x00\x00\x00\x00\x0089012345"
        )
        self.assertEqual(list(stream._recv_ranges), [range(8, 16)])
        self.assertEqual(stream._recv_buffer_start, 0)

        # add data at offset 0
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=0, data=b"01234567")),
            StreamDataReceived(data=b"0123456789012345", end_stream=False, stream_id=0),
        )
        self.assertEqual(bytes(stream._recv_buffer), b"")
        self.assertEqual(list(stream._recv_ranges), [])
        self.assertEqual(stream._recv_buffer_start, 16)

    def test_recv_offset_only(self):
        stream = QuicStream(stream_id=0)

        # add data at offset 0
        self.assertEqual(stream.add_frame(QuicStreamFrame(offset=0, data=b"")), None)
        self.assertEqual(bytes(stream._recv_buffer), b"")
        self.assertEqual(list(stream._recv_ranges), [])
        self.assertEqual(stream._recv_buffer_start, 0)

        # add data at offset 8
        self.assertEqual(stream.add_frame(QuicStreamFrame(offset=8, data=b"")), None)
        self.assertEqual(
            bytes(stream._recv_buffer), b"\x00\x00\x00\x00\x00\x00\x00\x00"
        )
        self.assertEqual(list(stream._recv_ranges), [])
        self.assertEqual(stream._recv_buffer_start, 0)

    def test_recv_already_fully_consumed(self):
        stream = QuicStream(stream_id=0)

        # add data at offset 0
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=0, data=b"01234567")),
            StreamDataReceived(data=b"01234567", end_stream=False, stream_id=0),
        )
        self.assertEqual(bytes(stream._recv_buffer), b"")
        self.assertEqual(list(stream._recv_ranges), [])
        self.assertEqual(stream._recv_buffer_start, 8)

        # add data again at offset 0
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=0, data=b"01234567")), None
        )
        self.assertEqual(bytes(stream._recv_buffer), b"")
        self.assertEqual(list(stream._recv_ranges), [])
        self.assertEqual(stream._recv_buffer_start, 8)

    def test_recv_already_partially_consumed(self):
        stream = QuicStream(stream_id=0)

        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=0, data=b"01234567")),
            StreamDataReceived(data=b"01234567", end_stream=False, stream_id=0),
        )

        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=0, data=b"0123456789012345")),
            StreamDataReceived(data=b"89012345", end_stream=False, stream_id=0),
        )
        self.assertEqual(bytes(stream._recv_buffer), b"")
        self.assertEqual(list(stream._recv_ranges), [])
        self.assertEqual(stream._recv_buffer_start, 16)

    def test_recv_fin(self):
        stream = QuicStream(stream_id=0)

        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=0, data=b"01234567")),
            StreamDataReceived(data=b"01234567", end_stream=False, stream_id=0),
        )
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=8, data=b"89012345", fin=True)),
            StreamDataReceived(data=b"89012345", end_stream=True, stream_id=0),
        )

    def test_recv_fin_out_of_order(self):
        stream = QuicStream(stream_id=0)

        # add data at offset 8 with FIN
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=8, data=b"89012345", fin=True)),
            None,
        )

        # add data at offset 0
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=0, data=b"01234567")),
            StreamDataReceived(data=b"0123456789012345", end_stream=True, stream_id=0),
        )

    def test_recv_fin_then_data(self):
        stream = QuicStream(stream_id=0)
        stream.add_frame(QuicStreamFrame(offset=0, data=b"", fin=True))
        with self.assertRaises(Exception) as cm:
            stream.add_frame(QuicStreamFrame(offset=0, data=b"01234567"))
        self.assertEqual(str(cm.exception), "Data received beyond FIN")

    def test_recv_fin_twice(self):
        stream = QuicStream(stream_id=0)
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=0, data=b"01234567")),
            StreamDataReceived(data=b"01234567", end_stream=False, stream_id=0),
        )
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=8, data=b"89012345", fin=True)),
            StreamDataReceived(data=b"89012345", end_stream=True, stream_id=0),
        )

        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=8, data=b"89012345", fin=True)),
            StreamDataReceived(data=b"", end_stream=True, stream_id=0),
        )

    def test_recv_fin_without_data(self):
        stream = QuicStream(stream_id=0)
        self.assertEqual(
            stream.add_frame(QuicStreamFrame(offset=0, data=b"", fin=True)),
            StreamDataReceived(data=b"", end_stream=True, stream_id=0),
        )

    def test_send_data(self):
        stream = QuicStream()
        self.assertEqual(stream.next_send_offset, 0)

        # nothing to send yet
        frame = stream.get_frame(8)
        self.assertIsNone(frame)

        # write data
        stream.write(b"0123456789012345")
        self.assertEqual(list(stream._send_pending), [range(0, 16)])
        self.assertEqual(stream.next_send_offset, 0)

        # send a chunk
        frame = stream.get_frame(8)
        self.assertEqual(frame.data, b"01234567")
        self.assertFalse(frame.fin)
        self.assertEqual(frame.offset, 0)
        self.assertEqual(list(stream._send_pending), [range(8, 16)])
        self.assertEqual(stream.next_send_offset, 8)

        # send another chunk
        frame = stream.get_frame(8)
        self.assertEqual(frame.data, b"89012345")
        self.assertFalse(frame.fin)
        self.assertEqual(frame.offset, 8)
        self.assertEqual(list(stream._send_pending), [])
        self.assertEqual(stream.next_send_offset, 16)

        # nothing more to send
        frame = stream.get_frame(8)
        self.assertIsNone(frame)
        self.assertEqual(list(stream._send_pending), [])
        self.assertEqual(stream.next_send_offset, 16)

        # first chunk gets acknowledged
        stream.on_data_delivery(QuicDeliveryState.ACKED, 0, 8)

        # second chunk gets acknowledged
        stream.on_data_delivery(QuicDeliveryState.ACKED, 8, 16)

    def test_send_data_and_fin(self):
        stream = QuicStream()

        # nothing to send yet
        frame = stream.get_frame(8)
        self.assertIsNone(frame)

        # write data and EOF
        stream.write(b"0123456789012345", end_stream=True)
        self.assertEqual(list(stream._send_pending), [range(0, 16)])
        self.assertEqual(stream.next_send_offset, 0)

        # send a chunk
        frame = stream.get_frame(8)
        self.assertEqual(frame.data, b"01234567")
        self.assertFalse(frame.fin)
        self.assertEqual(frame.offset, 0)
        self.assertEqual(stream.next_send_offset, 8)

        # send another chunk
        frame = stream.get_frame(8)
        self.assertEqual(frame.data, b"89012345")
        self.assertTrue(frame.fin)
        self.assertEqual(frame.offset, 8)
        self.assertEqual(stream.next_send_offset, 16)

        # nothing more to send
        frame = stream.get_frame(8)
        self.assertIsNone(frame)
        self.assertEqual(stream.next_send_offset, 16)

    def test_send_data_lost(self):
        stream = QuicStream()

        # nothing to send yet
        frame = stream.get_frame(8)
        self.assertIsNone(frame)

        # write data and EOF
        stream.write(b"0123456789012345", end_stream=True)
        self.assertEqual(list(stream._send_pending), [range(0, 16)])
        self.assertEqual(stream.next_send_offset, 0)

        # send a chunk
        self.assertEqual(
            stream.get_frame(8), QuicStreamFrame(data=b"01234567", fin=False, offset=0)
        )
        self.assertEqual(list(stream._send_pending), [range(8, 16)])
        self.assertEqual(stream.next_send_offset, 8)

        # send another chunk
        self.assertEqual(
            stream.get_frame(8), QuicStreamFrame(data=b"89012345", fin=True, offset=8)
        )
        self.assertEqual(list(stream._send_pending), [])
        self.assertEqual(stream.next_send_offset, 16)

        # nothing more to send
        self.assertIsNone(stream.get_frame(8))
        self.assertEqual(list(stream._send_pending), [])
        self.assertEqual(stream.next_send_offset, 16)

        # a chunk gets lost
        stream.on_data_delivery(QuicDeliveryState.LOST, 0, 8)
        self.assertEqual(list(stream._send_pending), [range(0, 8)])
        self.assertEqual(stream.next_send_offset, 0)

        # send chunk again
        self.assertEqual(
            stream.get_frame(8), QuicStreamFrame(data=b"01234567", fin=False, offset=0)
        )
        self.assertEqual(list(stream._send_pending), [])
        self.assertEqual(stream.next_send_offset, 16)

    def test_send_data_lost_fin(self):
        stream = QuicStream()

        # nothing to send yet
        frame = stream.get_frame(8)
        self.assertIsNone(frame)

        # write data and EOF
        stream.write(b"0123456789012345", end_stream=True)
        self.assertEqual(list(stream._send_pending), [range(0, 16)])
        self.assertEqual(stream.next_send_offset, 0)

        # send a chunk
        self.assertEqual(
            stream.get_frame(8), QuicStreamFrame(data=b"01234567", fin=False, offset=0)
        )
        self.assertEqual(list(stream._send_pending), [range(8, 16)])
        self.assertEqual(stream.next_send_offset, 8)

        # send another chunk
        self.assertEqual(
            stream.get_frame(8), QuicStreamFrame(data=b"89012345", fin=True, offset=8)
        )
        self.assertEqual(list(stream._send_pending), [])
        self.assertEqual(stream.next_send_offset, 16)

        # nothing more to send
        self.assertIsNone(stream.get_frame(8))
        self.assertEqual(list(stream._send_pending), [])
        self.assertEqual(stream.next_send_offset, 16)

        # a chunk gets lost
        stream.on_data_delivery(QuicDeliveryState.LOST, 8, 16)
        self.assertEqual(list(stream._send_pending), [range(8, 16)])
        self.assertEqual(stream.next_send_offset, 8)

        # send chunk again
        self.assertEqual(
            stream.get_frame(8), QuicStreamFrame(data=b"89012345", fin=True, offset=8)
        )
        self.assertEqual(list(stream._send_pending), [])
        self.assertEqual(stream.next_send_offset, 16)

    def test_send_blocked(self):
        stream = QuicStream()
        max_offset = 12

        # nothing to send yet
        frame = stream.get_frame(8, max_offset)
        self.assertIsNone(frame)
        self.assertEqual(list(stream._send_pending), [])
        self.assertEqual(stream.next_send_offset, 0)

        # write data, send a chunk
        stream.write(b"0123456789012345")
        frame = stream.get_frame(8)
        self.assertEqual(frame.data, b"01234567")
        self.assertFalse(frame.fin)
        self.assertEqual(frame.offset, 0)
        self.assertEqual(list(stream._send_pending), [range(8, 16)])
        self.assertEqual(stream.next_send_offset, 8)

        # send is limited by peer
        frame = stream.get_frame(8, max_offset)
        self.assertEqual(frame.data, b"8901")
        self.assertFalse(frame.fin)
        self.assertEqual(frame.offset, 8)
        self.assertEqual(list(stream._send_pending), [range(12, 16)])
        self.assertEqual(stream.next_send_offset, 12)

        # unable to send, blocked
        frame = stream.get_frame(8, max_offset)
        self.assertIsNone(frame)
        self.assertEqual(list(stream._send_pending), [range(12, 16)])
        self.assertEqual(stream.next_send_offset, 12)

        # write more data, still blocked
        stream.write(b"abcdefgh")
        frame = stream.get_frame(8, max_offset)
        self.assertIsNone(frame)
        self.assertEqual(list(stream._send_pending), [range(12, 24)])
        self.assertEqual(stream.next_send_offset, 12)

        # peer raises limit, send some data
        max_offset += 8
        frame = stream.get_frame(8, max_offset)
        self.assertEqual(frame.data, b"2345abcd")
        self.assertFalse(frame.fin)
        self.assertEqual(frame.offset, 12)
        self.assertEqual(list(stream._send_pending), [range(20, 24)])
        self.assertEqual(stream.next_send_offset, 20)

        # peer raises limit again, send remaining data
        max_offset += 8
        frame = stream.get_frame(8, max_offset)
        self.assertEqual(frame.data, b"efgh")
        self.assertFalse(frame.fin)
        self.assertEqual(frame.offset, 20)
        self.assertEqual(list(stream._send_pending), [])
        self.assertEqual(stream.next_send_offset, 24)

        # nothing more to send
        frame = stream.get_frame(8, max_offset)
        self.assertIsNone(frame)

    def test_send_fin_only(self):
        stream = QuicStream()

        # nothing to send yet
        self.assertTrue(stream.send_buffer_is_empty)
        frame = stream.get_frame(8)
        self.assertIsNone(frame)

        # write EOF
        stream.write(b"", end_stream=True)
        self.assertFalse(stream.send_buffer_is_empty)
        frame = stream.get_frame(8)
        self.assertEqual(frame.data, b"")
        self.assertTrue(frame.fin)
        self.assertEqual(frame.offset, 0)

        # nothing more to send
        self.assertFalse(stream.send_buffer_is_empty)  # FIXME?
        frame = stream.get_frame(8)
        self.assertIsNone(frame)
        self.assertTrue(stream.send_buffer_is_empty)

    def test_send_fin_only_despite_blocked(self):
        stream = QuicStream()

        # nothing to send yet
        self.assertTrue(stream.send_buffer_is_empty)
        frame = stream.get_frame(8)
        self.assertIsNone(frame)

        # write EOF
        stream.write(b"", end_stream=True)
        self.assertFalse(stream.send_buffer_is_empty)
        frame = stream.get_frame(8)
        self.assertEqual(frame.data, b"")
        self.assertTrue(frame.fin)
        self.assertEqual(frame.offset, 0)

        # nothing more to send
        self.assertFalse(stream.send_buffer_is_empty)  # FIXME?
        frame = stream.get_frame(8)
        self.assertIsNone(frame)
        self.assertTrue(stream.send_buffer_is_empty)
