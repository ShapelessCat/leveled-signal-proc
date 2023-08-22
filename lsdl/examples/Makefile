EXAMPLE_DIR_NAME=$(shell basename ${PWD})

all: cidr.json playtime.json eventcount.json app_combined.json

clean:
	rm -rf *.json

.PHONY: clean all

%.json: %.py
	@ echo "[LDSL] Generating IR from LDSL source $^"
	@ cd .. && python3 $(EXAMPLE_DIR_NAME)/$^ > $(EXAMPLE_DIR_NAME)/$@ || rm $(EXAMPLE_DIR_NAME)/$@
