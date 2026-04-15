# Root Makefile for Deespee Monorepo

.PHONY: install test lint deploy help local-infra dsp-run dmp-run run-exchange

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
	@echo "  make run-exchange Run the Go Ad Exchange simulator"

# --- Local Development ---

local-infra:
	@echo "Starting local infrastructure (Pub/Sub Emulator)..."
	docker-compose up -d

dsp-run:
	@echo "Running DSP service..."
	cargo run -p dsp

dmp-run:
	@echo "Running DMP service..."
	cargo run -p dmp

run-exchange:
	@echo "Running Ad Exchange Simulator (Go)..."
	cd adexchange && go run main.go

# --- Global Commands ---

install:
	@echo "Installing dependencies and setting up 'uv' virtual environment for agents..."
	cd agents && $(MAKE) install
	@echo "Setting up Rust workspace..."
	cargo build
	@echo "Setting up Go components (Ad Exchange)..."
	cd adexchange && go mod tidy

test: agents-test rust-test

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
