import os, sys, array, math, StringIO
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import subresource

class Image:
    """This class partially implements the interface of the PIL.Image.Image.
       One day in the future WPT might support the PIL module or another imaging
       library, so this hacky BMP implementation will no longer be required.
    """
    def __init__(self, width, height):
        self.width = width
        self.height = height
        self.img = bytearray([0 for i in range(3 * width * height)])

    @staticmethod
    def new(mode, size, color=0):
        return Image(size[0], size[1])

    def _int_to_bytes(self, number):
        packed_bytes = [0, 0, 0, 0]
        for i in range(4):
            packed_bytes[i] = number & 0xFF
            number >>= 8

        return packed_bytes

    def putdata(self, color_data):
        for y in range(self.height):
            for x in range(self.width):
                i = x + y * self.width
                if i > len(color_data) - 1:
                    return

                self.img[i * 3: i * 3 + 3] = color_data[i][::-1]

    def save(self, f, type):
        assert type == "BMP"
        # 54 bytes of preambule + image color data.
        filesize = 54 + 3 * self.width * self.height;
        # 14 bytes of header.
        bmpfileheader = bytearray(['B', 'M'] + self._int_to_bytes(filesize) +
                                  [0, 0, 0, 0, 54, 0, 0, 0])
        # 40 bytes of info.
        bmpinfoheader = bytearray([40, 0, 0, 0] +
                                  self._int_to_bytes(self.width) +
                                  self._int_to_bytes(self.height) +
                                  [1, 0, 24] + (25 * [0]))

        padlength = (4 - (self.width * 3) % 4) % 4
        bmppad = bytearray([0, 0, 0]);
        padding = bmppad[0 : padlength]

        f.write(bmpfileheader)
        f.write(bmpinfoheader)

        for i in range(self.height):
            offset = self.width * (self.height - i - 1) * 3
            f.write(self.img[offset : offset + 3 * self.width])
            f.write(padding)

def encode_string_as_bmp_image(string_data):
    data_bytes = array.array("B", string_data)
    num_bytes = len(data_bytes)

    # Convert data bytes to color data (RGB).
    color_data = []
    num_components = 3
    rgb = [0] * num_components
    i = 0
    for byte in data_bytes:
        component_index = i % num_components
        rgb[component_index] = byte
        if component_index == (num_components - 1) or i == (num_bytes - 1):
            color_data.append(tuple(rgb))
            rgb = [0] * num_components
        i += 1

    # Render image.
    num_pixels = len(color_data)
    sqrt = int(math.ceil(math.sqrt(num_pixels)))
    img = Image.new("RGB", (sqrt, sqrt), "black")
    img.putdata(color_data)

    # Flush image to string.
    f = StringIO.StringIO()
    img.save(f, "BMP")
    f.seek(0)

    return f.read()

def generate_payload(request, server_data):
    data = ('{"headers": %(headers)s}') % server_data
    if "id" in request.GET:
        request.server.stash.put(request.GET["id"], data)
    data = encode_string_as_bmp_image(data)
    return data

def generate_report_headers_payload(request, server_data):
    stashed_data = request.server.stash.take(request.GET["id"])
    return stashed_data

def main(request, response):
    handler = lambda data: generate_payload(request, data)
    content_type = 'image/bmp'

    if "report-headers" in request.GET:
        handler = lambda data: generate_report_headers_payload(request, data)
        content_type = 'application/json'

    subresource.respond(request,
                        response,
                        payload_generator = handler,
                        content_type = content_type,
                        access_control_allow_origin = "*")
