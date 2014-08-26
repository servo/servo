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

clean-embedding:
	@$(call E, "cleaning embedding")
	$(Q)cd $(B)/src/components/embedding/ && rm -rf libembedding*.dylib libembedding*.dSYM libembedding*.so $(DONE_embedding)

define DEF_CLEAN_SERVO_RULES

.PHONY: clean-$(1)
clean-$(1):
	@$$(call E, "cleaning $(1)")
	$$(Q)cd $$(B)/src/components/$(1)/ && rm -rf lib$(1)*.dylib lib$(1)*.rlib lib$(1)*.dSYM lib$(1)*.so $$(DONE_$(1))

endef

$(foreach lib_crate,$(SERVO_LIB_CRATES),$(eval $(call DEF_CLEAN_SERVO_RULES,$(lib_crate))))


clean-wpt:
	$(Q)rm -r _virtualenv
	$(Q)rm $(S)/src/test/wpt/metadata/MANIFEST.json

clean-servo: $(foreach lib_crate,$(SERVO_LIB_CRATES),clean-$(lib_crate))
	@$(call E, "cleaning servo")
	$(Q)rm -f servo servo-test $(foreach lib_crate,$(SERVO_LIB_CRATES),servo-test-$(lib_crate)) libservo*.so libservo*.a
	$(Q)cd $(BINDINGS_SRC) && rm -f *.pkl *.rs

clean-rust-snapshot-archives:
	@$(call E, "cleaning rust snapshot archives")
	$(Q)cd $(B)/rust_snapshot/ && rm -rf *.tgz
