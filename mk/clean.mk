define DEF_SUBMODULE_CLEAN_RULES
# clean target
clean-$(1) : 
	@$$(call E, make clean: $(1))
	$$(Q)rm -f $$(DONE_$(1))
	$$(Q)$$(MAKE) -C $$(B)src/$(1) clean

# add these targets to meta-targets
DEPS_CLEAN += clean-$(1)
endef

$(foreach submodule,$(CFG_SUBMODULES),\
$(eval $(call DEF_SUBMODULE_CLEAN_RULES,$(submodule))))

.PHONY:	clean $(DEPS_CLEAN)

clean: $(DEPS_CLEAN) clean-servo

clean-servo:
	rm -f servo servo-test
