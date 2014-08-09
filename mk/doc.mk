# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


RUSTDOC_HTML_IN_HEADER = $(S)/src/etc/rustdoc-style.html
RUSTDOC_FLAGS = --extern url=$(B)/src/support/url/rust-url/liburl.rlib --html-in-header $(RUSTDOC_HTML_IN_HEADER)
RUSTDOC_DEPS = $(RUSTDOC_HTML_IN_HEADER)

# FIXME(#2924) These crates make rustdoc fail for undetermined reasons.
DOC_BLACKLISTED := style


define DEF_SERVO_DOC_RULES
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
$(eval $(call DEF_SERVO_DOC_RULES,$(lib_crate))))


define DEF_SUBMODULES_DOC_RULES

ifeq (,$(filter $(1),$(DOC_BLACKLISTED)))

.PHONY: doc-$(1)
doc-$(1): $$(DONE_DEPS_$(1)) $$(ROUGH_DEPS_$(1)) $$(RUSTC_DEP_$(1))
	@$$(call E, rustdoc: $(1))
	$$(Q) \
	RUSTDOC_FLAGS="$$(ENV_RLDFLAGS_$(1)) $$(RUSTDOC_FLAGS)" \
	RUSTDOC_TARGET="$$(CFG_BUILD_HOME)/doc" \
	$$(ENV_EXT_DEPS_$(1)) \
	$$(MAKE) -C $$(B)src/$$(PATH_$(1)) doc

else

.PHONY: doc-$(1)
doc-$(1): $$(DONE_DEPS_$(1)) $$(ROUGH_DEPS_$(1)) $$(RUSTC_DEP_$(1))
	@echo SKIPPED: blacklisted rustdoc: $$@

endif
endef

# Only Rust submodules
DOC_SUBMODULES = $(foreach submodule,$(SUBMODULES),\
                   $(if $(RUSTC_DEP_$(submodule)), $(submodule)))


$(foreach submodule,$(DOC_SUBMODULES),\
$(eval $(call DEF_SUBMODULES_DOC_RULES,$(submodule))))


.PHONY: doc
doc: $(foreach target,$(DOC_SUBMODULES) $(SERVO_LIB_CRATES) servo,doc-$(target))
