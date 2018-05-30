"""
This generates a 30 minute silent wav, and is capable of
responding to Range requests.
"""
import time
import re
import struct


def create_wav_header(sample_rate, bit_depth, channels, duration):
    bytes_per_sample = bit_depth / 8
    block_align = bytes_per_sample * channels
    byte_rate = sample_rate * block_align
    sub_chunk_2_size = duration * byte_rate

    data = b''
    # ChunkID
    data += b'RIFF'
    # ChunkSize
    data += struct.pack('<L', 36 + sub_chunk_2_size)
    # Format
    data += b'WAVE'
    # Subchunk1ID
    data += b'fmt '
    # Subchunk1Size
    data += struct.pack('<L', 16)
    # AudioFormat
    data += struct.pack('<H', 1)
    # NumChannels
    data += struct.pack('<H', channels)
    # SampleRate
    data += struct.pack('<L', sample_rate)
    # ByteRate
    data += struct.pack('<L', byte_rate)
    # BlockAlign
    data += struct.pack('<H', block_align)
    # BitsPerSample
    data += struct.pack('<H', bit_depth)
    # Subchunk2ID
    data += b'data'
    # Subchunk2Size
    data += struct.pack('<L', sub_chunk_2_size)

    return data


def main(request, response):
    response.headers.set("Content-Type", "audio/wav")
    response.headers.set("Accept-Ranges", "bytes")
    response.headers.set("Cache-Control", "no-cache")

    range_header = request.headers.get('Range', '')
    range_received_key = request.GET.first('range-received-key', '')

    if range_received_key and range_header:
        # This is later collected using stash-take.py
        request.stash.put(range_received_key, 'range-header-received', '/fetch/range/')

    # Audio details
    sample_rate = 8000
    bit_depth = 8
    channels = 1
    duration = 60 * 5

    total_length = (sample_rate * bit_depth * channels * duration) / 8
    bytes_remaining_to_send = total_length
    initial_write = ''

    if range_header:
        response.status = 206
        start, end = re.search(r'^bytes=(\d*)-(\d*)$', range_header).groups()

        start = int(start)
        end = int(end) if end else 0

        if end:
            bytes_remaining_to_send = (end + 1) - start
        else:
            bytes_remaining_to_send = total_length - start

        wav_header = create_wav_header(sample_rate, bit_depth, channels, duration)

        if start < len(wav_header):
            initial_write = wav_header[start:]

            if bytes_remaining_to_send < len(initial_write):
                initial_write = initial_write[0:bytes_remaining_to_send]

        content_range = "bytes {}-{}/{}".format(start, end or total_length - 1, total_length)

        response.headers.set("Content-Range", content_range)
    else:
        initial_write = create_wav_header(sample_rate, bit_depth, channels, duration)

    response.headers.set("Content-Length", bytes_remaining_to_send)

    response.write_status_headers()
    response.writer.write(initial_write)

    bytes_remaining_to_send -= len(initial_write)

    while bytes_remaining_to_send > 0:
        if not response.writer.flush():
            break

        to_send = b'\x00' * min(bytes_remaining_to_send, sample_rate)
        bytes_remaining_to_send -= len(to_send)

        response.writer.write(to_send)
        # Throttle the stream
        time.sleep(0.5)
