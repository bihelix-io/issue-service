# Issue Service

A secure Bitcoin transaction signing service built with Rust, designed to provide PSBT (Partially Signed Bitcoin Transaction) signing capabilities for the transcription-service ecosystem.

## Features

- 🔐 **Secure Transaction Signing**: Sign Bitcoin transactions using extended private keys (xprv)
- 🌐 **Network Flexibility**: Support for Bitcoin mainnet, testnet, and regtest networks
- 🚀 **High Performance**: Built with Rust for optimal performance and memory safety
- 🔧 **Configurable**: Easy configuration through TOML files
- 🛡️ **Security First**: Designed with security best practices for private key management

## Table of Contents

- [Installation](#installation)
- [Configuration](#configuration)
- [Key Generation](#key-generation)
- [Usage](#usage)
- [Security](#security)
- [Contributing](#contributing)
- [License](#license)

## Installation

### Prerequisites

- Rust 1.70+ 
- Cargo package manager

### Build from Source

```bash
git clone https://github.com/your-username/issue-service.git
cd issue-service
cargo build --release
```

### Install BDK CLI (for key generation)

```bash
cargo install bdk-cli --version "0.27.1" --features esplora --locked
```

## Configuration

The service uses a `config.toml` file for configuration. Create one in your project root:

```toml
# Bitcoin network configuration
network = "bitcoin"  # Options: "bitcoin", "testnet", "regtest"

# Service port
port = 3001

# Extended private key for transaction signing
# WARNING: Keep this secure and never commit to version control
xprv = "your_extended_private_key_here"
```

### Configuration Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `network` | String | `"bitcoin"` | Bitcoin network type (bitcoin/testnet/regtest) |
| `port` | Integer | `3001` | HTTP server port |
| `xprv` | String | - | Extended private key for signing transactions |

## Key Generation

Before running the service, generate your cryptographic keys following RGB-44 specification:

### RGB-44 Derivation Paths

This service follows the RGB-44 specification ([RFC](https://github.com/RGB-WG/RFC/blob/master/RGB-44.md)) for key derivation paths:

**Mainnet (cointype 827166):**
- Receiving addresses: `84h/827166h/0h/0/*`
- Change addresses: `84h/827166h/0h/1/*`

**Testnet (cointype 827167):**
- Receiving addresses: `84h/827167h/0h/0/*`
- Change addresses: `84h/827167h/0h/1/*`

### Key Generation Steps

### Step 1: Generate Master Key
```bash
bdk-cli -n bitcoin key generate -e 12
```

This command outputs:
- Fingerprint
- Mnemonic phrase (12 words)
- Master private key (xprv)

### Step 2: Derive Extended Private Key
```bash
# For mainnet
bdk-cli -n bitcoin key derive -x "xprv_from_step_1" -p "84h/827166h/0h"

# For testnet
bdk-cli -n testnet key derive -x "xprv_from_step_1" -p "84h/827167h/0h"
```

### Step 3: Update Configuration
Copy the generated `xprv` to your `config.toml` file.

**Important:** Before using the xprv, you need to add `/0` before the final `/*` in the key path:

```
# Original output from Step 2:
[f643cd61/84'/827166'/0']xprv9yeSw9zd2GiZgErj9uMrWT963TAfpm2oZQmHC3Hwyxsq8s6SxkiQHAE5z4WKAXyQxYaVPTkK96ijVT81LhhfEn5iNnc5QzLACJSM8Wr13ER/*

# Modified for config.toml:
[f643cd61/84'/827166'/0']xprv9yeSw9zd2GiZgErj9uMrWT963TAfpm2oZQmHC3Hwyxsq8s6SxkiQHAE5z4WKAXyQxYaVPTkK96ijVT81LhhfEn5iNnc5QzLACJSM8Wr13ER/0/*
```

This modification ensures the correct derivation path for receiving addresses (`/0/*`) as specified in the RGB-44 standard.

## Usage

### Local Development

```bash
# Run with default configuration
cargo run config.toml

# Or specify custom config file
cargo run /path/to/your/config.toml
```

The service will be available at `http://127.0.0.1:3001` by default.

### Production Deployment

For production environments:

1. **Build optimized binary:**
   ```bash
   cargo build --release
   ```

2. **Deploy to your infrastructure:**
   ```bash
   # Copy binary to your server
   scp target/release/issue-service user@your-server:/usr/local/bin/
   
   # Run as a service (example with systemd)
   sudo systemctl start issue-service
   ```

3. **Configure your public URL:**
   Once deployed, your service will be accessible at your public domain (e.g., `https://your-domain.com`).

### Integration with Transcription Service

After deploying the issue-service, you need to configure it in the transcription service:

1. Navigate to the transcription service interface
2. Go to **Create Pool** → **Basic parameters** → **Sign API**
3. Enter your deployed issue-service URL (e.g., `https://your-domain.com`)

This URL will be used by the transcription service to communicate with your signing service for PSBT operations.

## Security

### 🔒 Private Key Management

- **Never commit private keys to version control**
- Store `xprv` in secure environment variables or key management systems
- Use hardware security modules (HSMs) for production deployments
- Regularly rotate keys and monitor access

### 🛡️ Best Practices

- Deploy behind reverse proxy with TLS termination
- Implement rate limiting and request validation
- Use network isolation and firewalls
- Enable audit logging for all signing operations
- Regular security audits and dependency updates

### 🚨 Important Security Notes

> **⚠️ WARNING:** This service handles private keys and should only be deployed in secure, controlled environments. Never expose this service directly to the public internet without proper security measures.

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Transcription   │───>│ Issue Service   │───>│ Bitcoin Network │
│ Service         │    │ (This Project)  │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Contributing

We welcome contributions! Here's how you can help improve this project:

### Development Setup

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Testing

```bash
# Run all tests
cargo test

# Run tests with coverage
cargo test --all-features
```

## Deployment Options

### Docker Deployment

```dockerfile
FROM rust:1.70-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/issue-service /usr/local/bin/
EXPOSE 3001
CMD ["issue-service", "config.toml"]
```

### Cloud Deployment

- **AWS**: Deploy on EC2 with Application Load Balancer
- **Google Cloud**: Use Cloud Run or Compute Engine
- **Azure**: Deploy on Container Instances or Virtual Machines

## FAQ

**Q: Can I use this service for mainnet transactions?**

A: Yes, but ensure you follow all security best practices and thoroughly test in testnet first.

**Q: How do I backup my keys?**

A: Store your mnemonic phrase securely offline. The extended private key can be regenerated from the mnemonic.

**Q: Is this service suitable for high-frequency trading?**

A: This service is optimized for reliable signing operations but may need additional optimization for high-frequency scenarios.

## License

This project is licensed under the MIT License.

**⚡ Built with Rust | 🔐 Security First | 🚀 Production Ready**
