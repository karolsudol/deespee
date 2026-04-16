# deespee

<img src="deespee.png" width="400" height="400" alt="Deespee logo">

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

## 🗺️ Roadmap & Execution Plan

### Phase 1: Core RTB & Memory (DMP) - **Phase Completed**
Establish the high-speed data link between the bidding muscle and the audience memory.
*   [x] **Workspace Setup:** Rust Monorepo with shared Protobufs.
*   [x] **Traffic Simulation:** High-performance Rust Ad Exchange.
*   [x] **DMP MVP:** High-speed user profile store with in-memory state.
*   [x] **Hot Path:** DSP ↔ DMP lookup for real-time segments (<10ms).
*   [x] **Frequency Capping:** Recency/Frequency tracking per user via Pub/Sub loop.
*   **Components:** `crates/proto`, `crates/dmp`, `crates/dsp`.

### Phase 2: Advanced Targeting & Bidding
Moving beyond simple bids to complex, multi-variable targeting.
*   [x] **Geo/IP Targeting:** City, Country, and ISP-based bidding.
*   [x] **Contextual Engine:** IAB category matching and domain blacklists.
*   [x] **Budget Pacing:** Smooth delivery algorithms (avoid spending daily budget in 5 mins).
*   [x] **Bidding Models:** CPM vs. eCPC optimized bidding.
*   **Components:** `crates/dsp`, `crates/dmp`.

### Phase 3: Measurement & Verification
The "Source of Truth" for what actually happened on the website.
*   [x] **Tracking Pixels:** Impression, Click, and Conversion event collectors.
*   [x] **Viewability:** IAB/MRC standard tracking (was the ad actually seen?).
*   [x] **Verification:** Bot detection and fraud filtering (SIVT/GIVT).
*   [x] **Discrepancy Engine:** Real-time reconciliation between DSP and Exchange stats.

*   **Components:** `crates/dsp`, `crates/collector`, `BigQuery`.


### Phase 4: Optimization & Learning
Using data to improve ROI through advanced modeling and testing.
*   [ ] **Attribution Models:** First-touch, Last-touch, and Multi-touch attribution (MTA).
*   [ ] **A/B Testing:** Multi-arm bandit testing for creatives and targeting strategies.
*   [ ] **DCO:** Dynamic Creative Optimization based on user attributes.
*   [ ] **Analytics Pipeline:** Real-time streaming from Pub/Sub to BigQuery/Looker.
*   **Components:** `BigQuery`, `Python/Agents` (Data Processing).

### Phase 5: Agentic Control & Interface (ADT)
The final stage: AI autonomously managing millions of dollars in spend.
*   [ ] **ADT Hub:** Agency Trading Desk API for campaign management.
*   [ ] **Autonomous Optimizer:** AI Agent adjusts bids/budgets based on ROI hourly.
*   [ ] **Human-Agent Chat:** Natural language interface for high-level strategy shifts.
*   **Components:** `agents/`, `crates/adt-api` (Upcoming).

> **Note on Storage:** For this demo, we use **Firestore** for simplicity and scale-to-zero. In a production RTB environment with <10ms requirements, **Google Cloud Memorystore (Redis)** is used for the hot path lookup between DSP and DMP.

## Project Structure

This is a monorepo containing multiple components:

- **`agents/`**: AI Agents powered by Google ADK (Python). Responsible for campaign strategy and optimization.
- **`crates/dsp`**: High-performance Demand-Side Platform (Rust).
- **`crates/dmp`**: Data Management Platform (Rust).
- **`crates/adexchange`**: Ad Exchange Simulator (Rust).
- **`crates/proto`**: Shared Protobuf schemas and generated code.
- **`shared/`**: Source Protobuf schemas.
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
3.  **Run the Ad Exchange Simulator (Rust):**
    ```bash
    make run-exchange
    ```
4.  **Run the Agent playground (Python):**
    ```bash
    cd agents && make playground
    ```

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
