import os, sys, array, Image, json, math, cStringIO
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import subresource

def encode_string_as_png_image(string_data):
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

    # Flush PNG to string.
    f = cStringIO.StringIO()
    img.save(f, "PNG")
    f.seek(0)

    return f.read()

def generate_payload(server_data):
    data = ('{"headers": %(headers)s}') % server_data
    return encode_string_as_png_image(data)

def main(request, response):
    subresource.respond(request,
                        response,
                        payload_generator = generate_payload,
                        content_type = "image/png",
                        access_control_allow_origin = "*")
