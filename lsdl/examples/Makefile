EXAMPLE_DIR_NAME=$(shell basename ${PWD})

all: cidr.json playtime.json event_count.json app_combined.json earliest_event_to_state.json

clean:
	rm -rf *.json

.PHONY: clean all

%.json: %.py
	@ echo "[LDSL] Generating IR from LDSL source $^"
	@ cd .. && python3 $(EXAMPLE_DIR_NAME)/$^ > $(EXAMPLE_DIR_NAME)/$@ || rm $(EXAMPLE_DIR_NAME)/$@
