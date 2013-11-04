EMPTY=
SPACE=$(EMPTY) $(EMPTY)

define DEF_SUBMODULE_RUSTPKG_RULES

$(eval $(call DEF_SUBMODULE_DEPS,$(1)))

$(1) : $$(DONE_$(1))
.PHONY : $(1)

DO_CLEAN_$(1) = rm -rf $$(DONE_$(1)) $(CFG_BUILD_HOME)/workspace/build/$(CFG_TARGET_TRIPLES)/$(1)

clean-$(1) :
	$$(Q) $$(DO_CLEAN_$(1))
.PHONY : clean-$(1)

# Need to clean otherwise rustpkg won't rebuild.
$$(DONE_$(1)) : $$(DONE_rust) $$(DONE_DEPS_$(1)) $$(ROUGH_DEPS_$(1))
	$$(Q) $$(DO_CLEAN_$(1))
	$$(Q) RUST_PATH=$(CFG_BUILD_HOME)workspace:$(subst $(SPACE),:,$(foreach submodule,$(strip $(CFG_SUBMODULES_RUSTPKG)),$(S)src/$(submodule))) \
	$(CFG_RUSTPKG) --rust-path-hack install $(CFG_RUSTC_FLAGS) $(1)

endef

