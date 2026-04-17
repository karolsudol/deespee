# deespee

<img src="deespee.img" width="800" height="450" alt="Deespee logo">

Demand-Side Platform (DSP) integrated with Data Management Platform (DMP) and AI Agents for Real-Time Bidding (RTB) optimization.

## 🏗️ Architecture

```text
                                     +---------------------------+
                                     |   AGENCY TRADING DESK     |
                                     |  (Human-Agent Interface)  |
                                     +-------------+-------------+
                                                   |
                                     +-------------v-------------+
                                     |      AI AGENT (BRAIN)     |
                                     |    (Strategic Orchestrator) |
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
    |   AD EXCHANGE (RUST)    |                                         |      MOCK WEBSITE        |
    |  (Traffic Simulator)    <-----------------------------------------+   (SSP / Publisher)      |
    +-------------------------+                                         +--------------------------+
```

## 🗺️ Roadmap & Feature Progress

### ✅ Completed Features (Core Backend - Rust)
*   **High-Speed Bidding Engine (DSP):** Real-time Protobuf-based bidding with support for CPM and eCPC models.
*   **Advanced Targeting:** Geo/IP, Contextual (IAB Categories), and Audience Segment matching.
*   **Audience Memory (DMP):** Real-time user profile store with frequency capping and segment management.
*   **Measurement & Verification:** Tracking pixels, Viewability (`IntersectionObserver`), and Bot Detection.
*   **Rust Lakehouse (DWH):**
    *   **Unified Ingestion:** Centralized collection of Wins, Impressions, Clicks, and Conversions.
    *   **Real-Time Buffering:** Memory-resident buffering of events with background Parquet flushing.
    *   **Deduplication:** Automatic filtering of duplicate pixel pings and event notifications.
    *   **Apache Iceberg:** Formal table format management for ACID transactions and schema evolution.
    *   **SQL Query Layer:** DataFusion-powered SQL interface for the AI Agent.

### 🚧 Phase 4: Performance Analytics & POC (Current)
*   **The Fusion Server (DWH):**
    - [x] **DataFusion SQL Endpoint:** Query interface for SQL execution against Parquet events.
    - [ ] **Star Schema Views:** SQL views joining events with campaign dimensions for ROI calculation.
*   **Agent Tools (Hands & Eyes):**
    - [ ] **Tool: `query_performance`**: Agent runs SQL to see ROI, spend, and win rates via DWH.
    - [ ] **Tool: `manage_campaign`**: Agent calls the DMP API to pause/resume or adjust budgets.
*   **The Optimization POC:**
    - [ ] **Closed-Loop Test:** Agent identifies a low-ROI campaign via DWH and automatically pauses it via DMP.

### 🤖 Phase 5: Agentic Control & Interface (TODO)
*   **Autonomous Optimization:**
    - [ ] **Bid Shading:** Agent dynamically adjusts bids to pay the minimum required to win (saving budget).
    - [ ] **Budget Reallocation:** Agent automatically moves funds from low-ROI to high-ROI campaigns hourly.
    - [ ] **Lookalike Discovery:** Agent identifies high-performing segments and suggests new targeting rules.
*   **Agency Interface (ADT):**
    - [ ] **Natural Language Briefing:** "Brief" the agent on goals (e.g., "Max conversions for under $10").
    - [ ] **Anomaly Detection:** Agent alerts humans via chat if win rates or spend spike unexpectedly.

## 🔄 System Behavior & Optimization Loop

```text
       +----------------+      (1) Bid Request      +----------------+
       |  AD EXCHANGE   | ------------------------> |      DSP       |
       |  (Simulator)   | <------------------------ |     (Rust)     |
       +-------^--------+      (2) Bid Response     +-------+--------+
               |                                            |
               | (3) Win Notice / Pixel Ping                | (4) Lookup
               |                                            v
       +-------v--------+                           +----------------+
       |   COLLECTOR    |                           |      DMP       |
       |    (Rust)      |                           |  (Audience)    |
       +-------+--------+                           +-------^--------+
               |                                            |
               | (5) Streaming Data                         | (8) Optimize
               v                                            |
       +----------------+      (6) SQL Query        +-------+--------+
       | FUSION SERVER  | <------------------------ |    AI AGENT    |
       | (DataFusion)   | ------------------------> |  (Optimizer)   |
       +-------+--------+      (7) Analytics Data   +----------------+
               |
               | (Raw Parquet Files)
               v
       +----------------+
       |   LAKEHOUSE    |
       | (data/events)  |
       +----------------+
```

### ❄️ Data Warehouse Flow (The Rust Lakehouse)
To ensure sub-millisecond bidding and high-performance analytics, we use a **Lakehouse** architecture:
1.  **Ingestion:** The **DWH Service** consumes events.
2.  **Buffering:** Events are converted into **Apache Arrow RecordBatches**.
3.  **Persistence:** Batches are flushed to **Apache Parquet** files managed by **Apache Iceberg** in the `data/lakehouse` directory.
4.  **Query Layer (The Fusion Server):** A dedicated query engine using **DataFusion** allows the Agent to run standard SQL across these Iceberg tables locally.

> **Note on Storage:** For this demo, we use **Firestore** for simplicity and scale-to-zero. In a production RTB environment with <10ms requirements, **Google Cloud Memorystore (Redis)** is used for the hot path lookup between DSP and DMP.

## Project Structure

This is a monorepo containing multiple components:

- **`agents/`**: AI Agents powered by Google ADK (Python). Responsible for campaign strategy and optimization.
- **`crates/dsp`**: High-performance Demand-Side Platform (Rust).
- **`crates/dmp`**: Data Management Platform (Rust).
- **`crates/collector`**: Real-time Measurement & Verification (Rust). Handles pixel tracking, bot detection, and win-reconciliation.
- **`crates/dwh`**: Data Warehouse & Fusion Server (Rust). Persists events into **Apache Iceberg** format (Parquet) and provides a DataFusion SQL interface for the Agent.
- **`crates/adexchange`**: Ad Exchange Simulator (Rust).
- **`crates/proto`**: Shared Protobuf schemas and generated code.
- **`shared/`**: Source Protobuf schemas.
- **`deployment/`**: Infrastructure as Code (Terraform) for GCP.

## Key RTB Components

### 1. The Hot Path (Bidding)
The DSP receives bid requests from an exchange. It performs a sub-millisecond lookup in **Redis** (managed by the DMP) to identify the user's segments and decides whether to bid.

### 2. The Feedback Loop
All win/loss notifications and clicks are streamed into the **Rust Lakehouse**.

### 3. The Optimization Loop (AI Agent)
The AI Agent queries the **Fusion Server** to analyze campaign performance (CTR, Spend, Win Rate). It then "optimizes" the campaign by updating the DMP's segments or the DSP's bidding strategy.

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

To run the full end-to-end demo locally, open separate terminal tabs for each service and run the following in order:

| Service / Action | Command |
| :--- | :--- |
| **1. Local Infrastructure** (Pub/Sub Emulator) | `make local-infra` |
| **2. DMP** (Audience & Campaigns) | `make dmp-run` |
| **3. DSP** (Bidding Engine) | `make dsp-run` |
| **4. Collector** (Measurement & Bot Detection) | `make collector-run` |
| **5. DWH** (Lakehouse & Query Server) | `make dwh-run` |
| **6. Ad Exchange** (Traffic Simulator) | `make adexchange-run` |
| **7. AI Agent** (Optimization Playground) | `cd agents && make playground` |

### Infrastructure Management
- **`make local-stop`**: Stop the Pub/Sub emulator.
- **`make local-down`**: Remove emulator containers.
- **`make local-clean`**: Remove containers and volumes (full reset).
- **`make local-restart`**: Quick restart of the infrastructure.
- **`make clean`**: Clean all build artifacts (Rust and Python).


## 🎮 Demo Walkthrough

Once all services are running, you can observe the following:

1.  **Bidding Flow:** The Go Simulator sends a **Binary Protobuf** `BidRequest` to the Rust DSP every 5 seconds.
2.  **DSP Decision:** The Rust DSP decodes the request, applies bidding logic, and responds with a binary `BidResponse`.
3.  **Agent Interaction:** In the Agent Playground, you can ask the AI Agent to "Trigger a DSP request". The Agent will publish a Protobuf `AgentRequest` to the local Pub/Sub emulator, which the DSP is configured to receive.
4.  **Monitoring Reconciliation:** You can view real-time discrepancy stats (Wins vs. Impressions) by visiting the Measurement Collector's report:
    - [Discrepancy Report (Local)](http://localhost:8003/report)

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
