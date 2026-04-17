# Root Makefile for Deespee Monorepo

.PHONY: install test lint deploy help local-infra dsp-run dmp-run collector-run dwh-run adexchange-run

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
	@echo "  make dsp-run    Run the Rust DSP service"
	@echo "  make dmp-run    Run the Rust DMP service"
	@echo "  make collector-run Run the Measurement Collector service"
	@echo "  make dwh-run    Run the DWH Lakehouse service"
	@echo "  make adexchange-run Run the Ad Exchange simulator"

# --- Local Development ---

local-infra:
	@echo "Starting local infrastructure (Pub/Sub Emulator)..."
	docker-compose up -d

local-stop:
	@echo "Stopping local infrastructure..."
	docker-compose stop

local-down:
	@echo "Removing local infrastructure containers..."
	docker-compose down

local-clean:
	@echo "Cleaning local infrastructure (containers and volumes)..."
	docker-compose down -v

local-restart: local-down local-infra

dsp-run:
	@echo "Running DSP service..."
	cargo run -p dsp

dmp-run:
	@echo "Running DMP service..."
	cargo run -p dmp

collector-run:
	@echo "Running Collector service..."
	cargo run -p collector

dwh-run:
	@echo "Running DWH service..."
	cargo run -p dwh

adexchange-run:
	@echo "Running Ad Exchange Simulator (Rust)..."
	cargo run -p adexchange

# --- Global Commands ---

install:
	@echo "Installing dependencies and setting up 'uv' virtual environment for agents..."
	cd agents && $(MAKE) install
	@echo "Setting up Rust workspace..."
	cargo build

clean:
	@echo "Cleaning Rust build artifacts..."
	cargo clean
	@echo "Cleaning Python artifacts..."
	find . -type d -name "__pycache__" -exec rm -rf {} +
	find . -type d -name ".pytest_cache" -exec rm -rf {} +
	find . -type d -name ".ruff_cache" -exec rm -rf {} +

rust-test:
	@echo "Running Rust workspace tests..."
	cargo test

lint: agents-lint rust-lint

rust-lint:
	@echo "Running Rust workspace linting..."
	cargo clippy && cargo fmt -- --check

# --- Agents Component ---

agents-test:
	@echo "Running tests for agents..."
	cd agents && $(MAKE) test

agents-lint:
	@echo "Running lint for agents..."
	cd agents && $(MAKE) lint

# --- Infrastructure ---

setup-dev-env:
	@echo "Setting up development environment infrastructure..."
	cd deployment/terraform/dev && terraform init && terraform apply -auto-approve
