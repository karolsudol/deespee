# Deespee

Demand-Side Platform (DSP) integrated with Data Management Platform (DMP) and AI Agents for Real-Time Bidding (RTB) optimization.

## 🏗️ Architecture

```text
                                     +---------------------------+
                                     |      USER INTERFACE       |
                                     |   (Agent Playground / UI) |
                                     +-------------+-------------+
                                                   |
                                     +-------------v-------------+
                                     |      AI AGENT (BRAIN)     |
                                     |    (Python / ADK Hub)     |
                                     +------+------+-------+-----+
                                            |      |       |
                 +--------------------------+      |       +--------------------------+
                 |                                 |                                  |
    +------------v------------+      +-------------v-------------+      +-------------v------------+
    |   CAMPAIGN ANALYTICS    |      |    STRATEGY & BIDDING     |      |    AUDIENCE SEGMENTS     |
    |      (BigQuery)         |      |      (DSP - Rust)         |      |      (DMP - Rust)        |
    +------------^------------+      +-------------+-------------+      +-------------+------------+
                 |                                 |                                  |
                 |                                 |          (Hot Path Lookup)       |
                 |                   +-------------v-------------+                    |
                 |                   |    REAL-TIME DATA STORE   |                    |
                 +-------------------+    (Firestore / Redis)    <--------------------+
                                     +-------------^-------------+
                                                   |
                                     +-------------+-------------+
                                     |    PUBSUB EVENT BUS       |
                                     |  (Wins / Clicks / Loss)   |
                                     +-------------^-------------+
                                                   |
                 +---------------------------------+----------------------------------+
                 |                                                                    |
    +------------+------------+                                         +-------------+------------+
    |   AD EXCHANGE (GO)      |                                         |      MOCK WEBSITE        |
    |  (Traffic Simulator)    <-----------------------------------------+   (SSP / Publisher)      |
    +-------------------------+                                         +--------------------------+
```

> **Note on Storage:** For this demo, we use **Firestore** for simplicity and scale-to-zero. In a production RTB environment with <10ms requirements, **Google Cloud Memorystore (Redis)** is used for the hot path lookup between DSP and DMP.

## Project Structure

This is a monorepo containing multiple components:

- **`agents/`**: AI Agents powered by Google ADK (Python). Responsible for campaign strategy and optimization.
- **`dsp/`**: High-performance Demand-Side Platform (Rust). Handles millisecond-latency bidding.
- **`dmp/`**: Data Management Platform (Rust). Manages user profiles and audience segments.
- **`adexchange/`**: Ad Exchange Simulator (Go). Generates bid requests and simulates traffic.
- **`shared/`**: Shared Protobuf schemas for cross-service communication.
- **`deployment/`**: Infrastructure as Code (Terraform) for GCP.

## Key RTB Components

### 1. The Hot Path (Bidding)
The DSP receives bid requests from an exchange. It performs a sub-millisecond lookup in **Redis** (managed by the DMP) to identify the user's segments and decides whether to bid.

### 2. The Feedback Loop
All win/loss notifications and clicks are streamed via **Pub/Sub** into **BigQuery**.

### 3. The Optimization Loop (AI Agent)
The AI Agent queries BigQuery to analyze campaign performance (CTR, Spend, Win Rate). It then "optimizes" the campaign by updating the DMP's segments or the DSP's bidding strategy.

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

To run the full end-to-end demo locally:

1.  **Start local infrastructure (Pub/Sub Emulator):**
    ```bash
    make local-infra
    ```
2.  **Run the DSP service (Rust):**
    ```bash
    make dsp-run
    ```
3.  **Run the Ad Exchange Simulator (Go):**
    ```bash
    make run-exchange
    ```
4.  **Run the Agent playground (Python):**
    ```bash
    cd agents && make playground
    ```

## 🎮 Demo Walkthrough

Once all services are running, you can observe the following:

1.  **Bidding Flow:** The Go Simulator sends a **Binary Protobuf** `BidRequest` to the Rust DSP every 5 seconds.
2.  **DSP Decision:** The Rust DSP decodes the request, applies bidding logic, and responds with a binary `BidResponse`.
3.  **Agent Interaction:** In the Agent Playground, you can ask the AI Agent to "Trigger a DSP request". The Agent will publish a Protobuf `AgentRequest` to the local Pub/Sub emulator, which the DSP is configured to receive.

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
