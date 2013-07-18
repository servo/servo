define DEF_SUBMODULE_TEST_RULES
# check target
.PHONY: check-$(1)
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
	$(RUSTC) -o $@ $<

contenttest: $(S)src/test/harness/contenttest/contenttest.rs servo
	$(RUSTC) $(RFLAGS_servo) -o $@ $< -L .


DEPS_CHECK_TESTABLE = $(filter-out $(NO_TESTS),$(DEPS_CHECK_ALL))
DEPS_CHECK_TARGETS_ALL = $(addprefix check-,$(DEPS_CHECK_TESTABLE))
DEPS_CHECK_TARGETS_FAST = $(addprefix check-,$(filter-out $(SLOW_TESTS),$(DEPS_CHECK_TESTABLE)))

.PHONY: check-test
check-test:
	echo $(DEPS_CHECK_TARGETS_ALL)

.PHONY: check
check: $(DEPS_CHECK_TARGETS_FAST) check-servo tidy

.PHONY: check-all
check-all: $(DEPS_CHECK_TARGETS_ALL) check-servo tidy

.PHONY: check-servo
check-servo: servo-test
	./servo-test

.PHONY: check-ref
check-ref: reftest
	./reftest $(S)src/test/ref/*.list

.PHONY: check-content
check-content: contenttest
	./contenttest --source-dir=$(S)src/test/html/content $(TESTNAME)

.PHONY: tidy
tidy: 
	python $(S)src/etc/tidy.py $(S)src
