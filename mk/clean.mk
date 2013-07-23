define DEF_SUBMODULE_CLEAN_RULES
# clean target
clean-$(1) : 
	@$$(call E, make clean: $(1))
	$$(Q)rm -f $$(DONE_$(1))
	$$(Q)$$(MAKE) -C $$(B)src/$$(PATH_$(1)) clean

# add these targets to meta-targets
DEPS_CLEAN_ALL += $(1)
endef

$(foreach submodule,$(SUBMODULES),\
$(eval $(call DEF_SUBMODULE_CLEAN_RULES,$(submodule))))

DEPS_CLEAN_TARGETS_ALL = $(addprefix clean-,$(DEPS_CLEAN_ALL))
DEPS_CLEAN_TARGETS_FAST = $(addprefix clean-,$(filter-out $(SLOW_BUILDS),$(DEPS_CLEAN_ALL)))

.PHONY:	clean $(DEPS_CLEAN_TARGETS_ALL)

clean: $(DEPS_CLEAN_TARGETS_ALL) clean-servo
	@$(call E, "cleaning:")
	@$(call E, "    $(DEPS_CLEAN_ALL)")

clean-fast: $(DEPS_CLEAN_TARGETS_FAST) clean-servo
	@$(call E, "cleaning:")
	@$(call E, "    $(filter-out $(SLOW_BUILDS),$(DEPS_CLEAN_ALL))")

clean-util:
	@$(call E, "cleaning util")
	$(Q)cd $(B)/src/components/util/ && rm -rf libutil*.dylib libutil*.so $(DONE_util)

clean-msg:
	@$(call E, "cleaning msg")
	$(Q)cd $(B)/src/components/msg/ && rm -rf libmsg*.dylib libmsg*.so $(DONE_msg)

clean-net:
	@$(call E, "cleaning net")
	$(Q)cd $(B)/src/components/net/ && rm -rf libnet*.dylib libnet*.so $(DONE_net)

clean-gfx:
	@$(call E, "cleaning gfx")
	$(Q)cd $(B)/src/components/gfx/ && rm -rf libgfx*.dylib libgfx*.so $(DONE_gfx)

clean-script:
	@$(call E, "cleaning script")
	$(Q)cd $(B)/src/components/script/ && rm -rf libscript*.dylib libscript*.so $(DONE_script)

clean-servo: clean-gfx clean-util clean-net clean-script clean-msg
	@$(call E, "cleaning servo")
	$(Q)rm -f servo servo-test
