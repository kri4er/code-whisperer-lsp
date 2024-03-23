##AWS Code whisperer LSP written in Rust!


### Prerequisites

### Installation

```bash
cat service-2.json | jq '.metadata += {"serviceId":"codewhisperer"}' | tee /tmp/aws-coral-model.json
```

### TODO:
* Make build.rs file to automate aws CLI service model installation
