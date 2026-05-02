# Validation harness for a model-generated push_swap binary.
#
# Usage (typically via validate.py inside docker):
#   make PUSH_SWAP=/path/to/push_swap test
#
# Just runs the Python tester against the binary; no compilation needed.

PUSH_SWAP ?= $(error PUSH_SWAP is unset. Pass PUSH_SWAP=/abs/or/relative/path)
PYTHON    ?= python3

.PHONY: test
test:
	@echo "==> running test_push_swap.py against $(PUSH_SWAP)"
	$(PYTHON) test_push_swap.py $(PUSH_SWAP)
