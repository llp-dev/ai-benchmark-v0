PYTHON ?= python3
DOCKER ?= docker

OPENCODE_IMAGE := ai-benchmark-opencode
VALIDATE_IMAGE := ai-benchmark-validate

.PHONY: help image validate-image run validate status

help:
	@echo "Targets:"
	@echo "  make image           build the opencode benchmark image ($(OPENCODE_IMAGE))"
	@echo "  make validate-image  build the validation image ($(VALIDATE_IMAGE))"
	@echo "  make run             run the benchmark over benchmark/config.json"
	@echo "  make validate        validate every slug under benchmark/, print summary table"
	@echo "  make status          show running benchmark containers"

image:
	$(DOCKER) build -t $(OPENCODE_IMAGE) .

validate-image:
	$(DOCKER) build -t $(VALIDATE_IMAGE) validation/

run:
	$(PYTHON) benchmark.py

validate: validate-image
	$(PYTHON) validation/run_all.py

status:
	@$(DOCKER) ps --filter name=benchmark- \
		--format 'table {{.Names}}\t{{.Status}}\t{{.RunningFor}}' \
		| (read -r header && echo "$$header" && cat) || true
	@$(DOCKER) ps --filter name=benchmark- --quiet | grep -q . \
		|| echo "(no benchmark containers running)"

.DEFAULT_GOAL := help
