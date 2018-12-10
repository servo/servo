def main(request, response):
    return "PASS" if request.POST["file_input"].file.read() == b"File to upload\n" else "FAIL"
