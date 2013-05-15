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
	$(Q)echo "Cleaning targets:"
	$(Q)echo "$(DEPS_CLEAN_ALL)"

clean-fast: $(DEPS_CLEAN_TARGETS_FAST) clean-servo
	$(Q)echo "Cleaning targets:"
	$(Q)echo "$(filter-out $(SLOW_BUILDS),$(DEPS_CLEAN_ALL))"

clean-servo-util:
	cd $(B)/src/components/servo-util/ && rm -rf libservo_util*.dylib $(DONE_servo_util)

clean-servo-net:
	cd $(B)/src/components/servo-net/ && rm -rf libservo_net*.dylib $(DONE_servo_net)

clean-servo-gfx:
	cd $(B)/src/components/servo-gfx/ && rm -rf libservo_gfx*.dylib $(DONE_servo_gfx)

clean-servo: clean-servo-gfx clean-servo-util clean-servo-net
	rm -f servo servo-test
