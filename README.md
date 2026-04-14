# Deespee

Demand-Side Platform (DSP) integrated with Data Management Platform (DMP) and AI Agents.

## Project Structure

This is a monorepo containing multiple components:

- **`agents/`**: AI Agents powered by Google ADK (Python).
- **`dsp/`**: Demand-Side Platform (Upcoming Rust implementation).
- **`dmp/`**: Data Management Platform (Upcoming Rust implementation).
- **`shared/`**: Shared data contracts and schemas (Protobuf/JSON Schema).
- **`deployment/`**: Unified infrastructure management using Terraform.
- **`.cloudbuild/`**: CI/CD pipeline configurations for Google Cloud Build.

## Getting Started

### Prerequisites
- **uv**: Python package manager.
- **Rust/Cargo**: For DSP and DMP components.
- **Terraform**: For infrastructure.
- **Google Cloud SDK**: For cloud resource management.

### Orchestration
Use the root-level `Makefile` to manage the entire project:

```bash
# Install all dependencies
make install

# Run all tests
make test

# Set up development infrastructure
make setup-dev-env
```

## Component Documentation
Each component has its own documentation within its respective directory.
- [Agents README](agents/README.md)
- [Deployment README](deployment/README.md)
