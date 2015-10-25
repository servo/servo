import os
def main(request, response):
    with open('./resources/ahem/AHEM____.TTF') as f:
        return 200, [], f.read()
