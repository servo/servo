define DEF_SUBMODULE_TEST_RULES
# check target
check-$(1) : $$(DONE_$(1))
	@$$(call E, make check: $(1))

	$$(Q) \
	$$(ENV_CFLAGS_$(1)) \
	$$(ENV_RFLAGS_$(1)) \
	$$(MAKE) -C $$(B)src/$$(PATH_$(1)) check

DEPS_CHECK_ALL += $(1)
endef

$(foreach submodule,$(SUBMODULES),\
$(eval $(call DEF_SUBMODULE_TEST_RULES,$(submodule))))


# Testing targets

servo-test: $(DEPS_servo)
	$(RUSTC) $(RFLAGS_servo) --test -o $@ $<

reftest: $(S)src/test/harness/reftest/reftest.rs servo
	$(RUSTC) $(RFLAGS_servo) -o $@ $< -L .

contenttest: $(S)src/test/harness/contenttest/contenttest.rs servo
	$(RUSTC) $(RFLAGS_servo) -o $@ $< -L .

DEPS_CHECK_TARGETS_ALL = $(addprefix check-,$(DEPS_CHECK_ALL))
DEPS_CHECK_TARGETS_FAST = $(addprefix check-,$(filter-out $(SLOW_TESTS),$(DEPS_CHECK_ALL)))

.PHONY: check $(DEPS_CHECK_TARGETS_ALL)

check: $(DEPS_CHECK_TARGETS_FAST) check-servo tidy

check-all: $(DEPS_CHECK_TARGETS_ALL) check-servo tidy

check-servo: servo-test
	./servo-test $(TESTNAME)

check-ref: reftest
	./reftest --source-dir=$(S)/src/test/html/ref --work-dir=src/test/html/ref $(TESTNAME)

check-content: contenttest
	./contenttest --source-dir=$(S)/src/test/content $(TESTNAME)

tidy: 
	python $(S)src/etc/tidy.py $(S)src
