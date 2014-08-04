# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


RUSTDOC_HTML_IN_HEADER = $(S)/src/etc/rustdoc-style.html
RUSTDOC_FLAGS = --html-in-header $(RUSTDOC_HTML_IN_HEADER)
RUSTDOC_DEPS = $(RUSTDOC_HTML_IN_HEADER)

# FIXME(#2924) These crates make rustdoc fail for undetermined reasons.
DOC_BLACKLISTED := style

define DEF_DOC_RULES
.PHONY: doc-$(1)
doc-$(1): doc/$(1)/index.html

ifeq (,$(filter $(1),$(DOC_BLACKLISTED)))

doc/$(1)/index.html: $$(DEPS_$(1)) $$(RUSTDOC_DEPS)
	@$$(call E, rustdoc: $$@)
	$$(Q)$$(RUSTDOC) $$(RUSTDOC_FLAGS) $$(RFLAGS_$(1)) $$<

else

.PHONY: doc/$(1)/index.html
doc/$(1)/index.html:
	@echo SKIPPED: blacklisted rustdoc: $$@

endif
endef

$(foreach lib_crate,$(SERVO_LIB_CRATES) servo,\
$(eval $(call DEF_DOC_RULES,$(lib_crate))))


.PHONY: doc
doc: $(foreach crate,$(SERVO_LIB_CRATES) servo,doc-$(crate))
