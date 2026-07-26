#!/bin/bash

BROTLI=brotli
MIMETYPE=mimetype
OPENSSL=openssl
ZSTD=zstd

all: \
  self-compressed-image-001.png.dcb \
  self-compressed-image-001.png.dcz \
  self-compressed-style-001.css.dcb \
  self-compressed-style-001.css.dcz \
  self-compressed-script-001.js.dcb \
  self-compressed-script-001.js.dcz \
  self-compressed-subframe-001.html.dcb \
  self-compressed-subframe-001.html.dcz \
  subframe-001-compressed-by-style-001.html.dcb \
  subframe-001-compressed-by-style-001.html.dcz \
  subframe-001-compressed-by-script-001.html.dcb \
  subframe-001-compressed-by-script-001.html.dcz

.PHONY: hashes

define generate_dcb
	$(eval in=$(1))
	$(eval dict=$(2))
	$(eval out=$(3))
	echo 'Cache-Control: max-age=3600' > $(in).headers
	echo 'Vary: available-dictionary,accept-encoding' >> $(in).headers
	echo 'Content-Encoding: dcb' > $(out).headers
	echo -n 'Content-Type: ' >> $(out).headers
	$(MIMETYPE) -b $(in) >> $(out).headers
	cp dcb-header $(out)
	$(OPENSSL) dgst -sha256 -binary $(dict) >> $(out)
	$(BROTLI) --stdout -D $(dict) $(in) >> $(out)
endef

define generate_dcz
	$(eval in=$(1))
	$(eval dict=$(2))
	$(eval out=$(3))
	echo 'Cache-Control: max-age=3600' > $(in).headers
	echo 'Vary: available-dictionary,accept-encoding' >> $(in).headers
	echo 'Content-Encoding: dcz' > $(out).headers
	echo -n 'Content-Type: ' >> $(out).headers
	$(MIMETYPE) -b $(in) >> $(out).headers
	cp dcz-header $(out)
	cp $(dict) /tmp/dcz.dict
	$(OPENSSL) dgst -sha256 -binary /tmp/dcz.dict >> $(out)
	$(ZSTD) -D /tmp/dcz.dict -f -o /tmp/dcz.zstd $(in)
	cat /tmp/dcz.zstd >> $(out)
endef

clean:
	rm -f *compressed* *.headers

self-compressed-%.dcb: %
	$(call generate_dcb,$<,$<,$@)

self-compressed-%.dcz: %
	$(call generate_dcz,$<,$<,$@)

subframe-001-compressed-by-style-001.html.dcb: subframe-001.html style-001.css
	$(call generate_dcb,$<,$(word 2,$^),$@)

subframe-001-compressed-by-style-001.html.dcz: subframe-001.html style-001.css
	$(call generate_dcz,$<,$(word 2,$^),$@)

subframe-001-compressed-by-script-001.html.dcb: subframe-001.html script-001.js
	$(call generate_dcb,$<,$(word 2,$^),$@)

subframe-001-compressed-by-script-001.html.dcz: subframe-001.html script-001.js
	$(call generate_dcz,$<,$(word 2,$^),$@)

# Not used by the tests, but just in case one needs it...
hashes: style-001.css script-001.js subframe-001.html image-001.png
	@echo "{"
	@for file in $^; do \
	    echo -n '  "'$$file'": ":'; \
	    cat $$file | $(OPENSSL) dgst -binary -sha256  | $(OPENSSL) base64 -A; \
	    echo ':",'; \
	    done
	@echo "}"
