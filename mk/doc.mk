# These crates make rustdoc fail for undetermined reasons.
DOC_BLACKLISTED := style layout

define DEF_DOC_RULES
.PHONY: doc-$(1)
doc-$(1): doc/$(1)/index.html

ifeq (,$(findstring $(1),$(DOC_BLACKLISTED)))

doc/$(1)/index.html: $$(DEPS_$(1))
	@$$(call E, rustdoc: $$@)
	$$(Q)$$(RUSTDOC) $$(RFLAGS_$(1)) $$<

else

.PHONY: doc/$(1)/index.html
doc/$(1)/index.html: $$(DEPS_$(1))
	@echo SKIPPED: blacklisted rustdoc: $$@

endif
endef

$(foreach lib_crate,$(SERVO_LIB_CRATES) servo,\
$(eval $(call DEF_DOC_RULES,$(lib_crate))))


.PHONY: doc
doc: $(foreach crate,$(SERVO_LIB_CRATES) servo,doc-$(crate))
