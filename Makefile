SOURCES := $(shell find src -name '*.c') # Get all .c files in src/

Hydra: $(SOURCES)
	clang $(SOURCES)