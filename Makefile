# Shortcuts for the daily commands of the stack, until a cleaner way replaces them: ai-ctl's own
# commands, and OpenWebUI's container, which ai-ctl does not manage (docs/stack.md §7.3).
# `make` alone lists the targets.

BIN := target/release/ai-ctl

# A model of the inventory, for `make logs MODEL=<id>`.
MODEL ?=

WEBUI_NAME   ?= openwebui
WEBUI_IMAGE  ?= ghcr.io/open-webui/open-webui:main
WEBUI_VOLUME ?= open-webui
WEBUI_PORT   ?= 18000

# Published on loopback only (docs/security.md §6); the container reaches the gateway at
# http://10.0.2.2:8080/v1 through allow_host_loopback.
WEBUI_RUN = podman run -d --name $(WEBUI_NAME) -p 127.0.0.1:$(WEBUI_PORT):8080 \
	--network=slirp4netns:allow_host_loopback=true \
	-v $(WEBUI_VOLUME):/app/backend/data $(WEBUI_IMAGE)

.DEFAULT_GOAL := help
.PHONY: help bin doctor up down restart status logs llama-update \
	webui-up webui-down webui-update webui-logs start stop

help: ## List the targets
	@awk 'BEGIN { FS = ":.*## " } /^[a-z-]+:.*## / { printf "  %-14s %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

$(BIN):
	cargo build --release

bin: ## Build ai-ctl again from the working tree
	cargo build --release

doctor: $(BIN) ## Check everything the stack needs on this machine
	$(BIN) doctor

up: $(BIN) ## Start the gateway; it starts llama-server for each model it loads
	$(BIN) up

down: $(BIN) ## Stop the gateway and its models
	$(BIN) down

restart: $(BIN) ## Stop then start the gateway, to apply a changed inventory, build or release
	$(BIN) down
	$(BIN) up

# `status` exits 3 when no gateway runs: an answer, not a failure.
status: $(BIN) ## Show the gateway, its models and the GPU memory
	@$(BIN) status || [ $$? -eq 3 ]

logs: $(BIN) ## Follow the gateway's log, or one model's output with MODEL=<id>
	$(BIN) logs $(if $(MODEL),$(MODEL),--follow)

llama-update: $(BIN) ## Build the newest llama.cpp release; `make restart` serves it
	$(BIN) build llama.cpp

webui-up: ## Start OpenWebUI, creating its container the first time
	@if podman container exists $(WEBUI_NAME); then \
		podman start $(WEBUI_NAME); \
	else \
		$(WEBUI_RUN); \
	fi
	@echo "OpenWebUI on http://127.0.0.1:$(WEBUI_PORT)"

webui-down: ## Stop OpenWebUI; its container and volume stay
	podman stop --ignore $(WEBUI_NAME)

# The chats and settings live in the volume, which the new container mounts again.
webui-update: ## Pull OpenWebUI's image, and recreate its container when the image changed
	podman pull $(WEBUI_IMAGE)
	@if [ "$$(podman container inspect --format '{{.Image}}' $(WEBUI_NAME) 2>/dev/null)" \
		= "$$(podman image inspect --format '{{.Id}}' $(WEBUI_IMAGE))" ]; then \
		echo "OpenWebUI runs the newest image already"; \
	else \
		podman rm -f --ignore $(WEBUI_NAME) && $(WEBUI_RUN); \
	fi

webui-logs: ## Follow OpenWebUI's output
	podman logs -f --tail 100 $(WEBUI_NAME)

start: up webui-up ## Start the gateway, then OpenWebUI

stop: webui-down down ## Stop OpenWebUI, then the gateway
