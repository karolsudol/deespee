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
- **Docker**: For local infrastructure (Pub/Sub emulator).
- **Terraform**: For infrastructure.
- **Google Cloud SDK**: For cloud resource management.

### Initial Setup
1.  **Clone the repository.**
2.  **Set up environment variables:**
    ```bash
    cp .env.example .env
    # Edit .env with your Google Cloud Project ID and desired region
    ```
3.  **Set up Terraform variables:**
    ```bash
    cp deployment/terraform/dev/vars/env.tfvars.example deployment/terraform/dev/vars/env.tfvars
    # Edit env.tfvars with your Google Cloud Project ID
    ```
4.  **Install dependencies:**
    ```bash
    make install
    ```

### Local Development Loop
1.  **Start local infrastructure (Pub/Sub Emulator):**
    ```bash
    make local-infra
    ```
2.  **Run the DSP service (Rust):**
    ```bash
    make dsp-run
    ```
3.  **Run the Agent playground (Python):**
    ```bash
    cd agents && make playground
    ```

### Orchestration
Use the root-level `Makefile` to manage the entire project:

```bash
# Run all tests
make test

# Set up development infrastructure (Terraform)
make setup-dev-env
```

## Component Documentation
Each component has its own documentation within its respective directory.
- [Agents README](agents/README.md)
- [Deployment README](deployment/README.md)
