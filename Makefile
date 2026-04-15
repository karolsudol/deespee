# Root Makefile for Deespee Monorepo

.PHONY: install test lint deploy help

# Default target
help:
	@echo "Deespee Monorepo Management"
	@echo ""
	@echo "Usage:"
	@echo "  make install    Install dependencies for all components"
	@echo "  make test       Run tests for all components"
	@echo "  make lint       Run linting for all components"
	@echo "  make deploy     Deploy infrastructure and services"
	@echo ""
	@echo "Component targets:"
	@echo "  make agents-test    Run tests for the agents component"
	@echo "  make dsp-test       Run tests for the DSP component (Rust)"
	@echo "  make dmp-test       Run tests for the DMP component (Rust)"

# --- Local Development ---

local-infra:
	@echo "Starting local infrastructure (Pub/Sub Emulator)..."
	docker-compose up -d

dsp-run:
	@echo "Running DSP service..."
	cd dsp && cargo run

# --- Global Commands ---

install:
	@echo "Installing dependencies and setting up 'uv' virtual environment for agents..."
	cd agents && $(MAKE) install
	@echo "Setting up Rust components (DSP/DMP)..."
	# Rust components use standard cargo, which handles its own environments

test: agents-test dsp-test dmp-test

lint: agents-lint dsp-lint dmp-lint

# --- Agents Component ---

agents-test:
	@echo "Running tests for agents..."
	cd agents && $(MAKE) test

agents-lint:
	@echo "Running lint for agents..."
	cd agents && $(MAKE) lint

# --- DSP Component (Rust) ---

dsp-test:
	@if [ -d "dsp" ]; then \
		echo "Running tests for DSP..."; \
		cd dsp && cargo test; \
	else \
		echo "DSP component not found, skipping..."; \
	fi

dsp-lint:
	@if [ -d "dsp" ]; then \
		echo "Running lint for DSP..."; \
		cd dsp && cargo clippy; \
	else \
		echo "DSP component not found, skipping..."; \
	fi

# --- DMP Component (Rust) ---

dmp-test:
	@if [ -d "dmp" ]; then \
		echo "Running tests for DMP..."; \
		cd dmp && cargo test; \
	else \
		echo "DMP component not found, skipping..."; \
	fi

dmp-lint:
	@if [ -d "dmp" ]; then \
		echo "Running lint for DMP..."; \
		cd dmp && cargo clippy; \
	else \
		echo "DMP component not found, skipping..."; \
	fi

# --- Infrastructure ---

setup-dev-env:
	@echo "Setting up development environment infrastructure..."
	cd deployment/terraform/dev && terraform init && terraform apply -auto-approve
