RAND := $(shell awk 'BEGIN{srand();printf("%d", 65536*rand())}')
CHAPTER ?=
# This is only for TAs to run base test, students do not know the token
TOKEN_SHA1 := $(shell echo -n '$(passwd)' | sha1sum | xargs | awk -F' ' '{ print $$1 }')
ifeq ($(TOKEN_SHA1), cb7997e364d3af0192a37edebaf166c0ecf10454)
	BASE := 1
	BASE_CHAR := b
else
	CHECK_REPORTS := 1
	BASE := 2
endif

# For students, report will be counted in to pass the CI
ifeq ($(CHAPTER), 3)
	LAB := 1
else ifeq ($(CHAPTER), 4)
	LAB := 2
else ifeq ($(CHAPTER), 5)
	INITPROC := 1
	LAB := 3
else ifeq ($(CHAPTER), 6)
	INITPROC := 1
	LAB := 4
else ifeq ($(CHAPTER), 7)
# For ch7 we have no ch7_usertest, so just run ch7b_usertest to pass the CI
	INITPROC := 1
	BASE_CHAR := b
	LAB := 4
else ifeq ($(CHAPTER), 8)
	INITPROC := 1
	LAB := 5
else ifeq ($(CHAPTER), 9)
	INITPROC := 1
	LAB := 6
endif

env:
	rustup uninstall nightly && rustup install nightly
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add riscv64gc-unknown-none-elf
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

randomize:
	find user/src/bin -name "*.rs" | xargs -I {} sh -c 'sed -i.bak 's/OK/OK$(RAND)/g' {} && rm -rf {}.bak'
	find user/src/bin -name "*.rs" | xargs -I {} sh -c 'sed -i.bak 's/passed/passed$(RAND)/g' {} && rm -rf {}.bak'
	find check -name "*.py" | xargs -I {} sh -c 'sed -i.bak 's/OK/OK$(RAND)/g' {} && rm -rf {}.bak'
	find check -name "*.py" | xargs -I {} sh -c 'sed -i.bak 's/passed/passed$(RAND)/g' {} && rm -rf {}.bak'

#test: env randomize
test: randomize
	python3 overwrite.py $(CHAPTER)
	make -C user build BASE=$(BASE) TEST=$(CHAPTER) CHAPTER=$(CHAPTER)
ifdef INITPROC
	cp -f user/build/elf/ch$(CHAPTER)$(BASE_CHAR)_usertest.elf user/build/elf/ch$(CHAPTER)b_initproc.elf
endif
	make -C ../os run | tee stdout-ch$(CHAPTER)
ifdef LAB
	python3 check/ch$(CHAPTER)$(BASE_CHAR).py < stdout-ch$(CHAPTER)
endif

ifneq ($(and $(LAB),$(CHECK_REPORTS)),)
	@for i in $(shell seq $(LAB)); do \
		if ! [ -f ../reports/lab$$i.pdf -o -f ../reports/lab$$i.md ]; then \
			echo "Report for lab$$i needed. Add your report to reports/lab$$i.pdf or reports/lab$$i.md" ; \
			exit 1 ; \
		else \
			echo "Report for lab$$i found." ; \
		fi; \
	done
endif

.PHONY: test
