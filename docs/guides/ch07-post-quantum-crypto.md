# Chapter 7: Post-Quantum Cryptography

> Hybrid cryptographic primitives, key exchange, signatures, and secure transport

---

## Why PQC Matters

Classical cryptographic algorithms (RSA, ECC, Diffie-Hellman) rely on mathematical problems that quantum computers can solve efficiently using Shor's algorithm. Post-Quantum Cryptography (PQC) uses algorithms based on problems believed to be hard for both classical and quantum computers.

### The Threat Model

| Algorithm | Classical Security | Quantum Threat |
|-----------|-------------------|----------------|
| RSA-2048 | Secure | Broken by Shor's |
| ECDH (P-256) | Secure | Broken by Shor's |
| AES-256 | Secure | Reduced to AES-128 (Grover's) |
| ML-KEM-768 | Secure | Secure against quantum |
| ML-DSA-65 | Secure | Secure against quantum |

### Fusion's Approach: Hybrid Cryptography

Fusion uses a **hybrid approach** — combining classical and post-quantum algorithms. This ensures:
1. Security against classical attacks (even if PQC has undiscovered weaknesses)
2. Security against quantum attacks (even if classical crypto is broken)
3. Backward compatibility with existing systems

---

## Hybrid Key Exchange (X25519 + ML-KEM-768)

Key exchange establishes a shared secret between two parties. Fusion combines X25519 (classical) with ML-KEM-768 (post-quantum).

### How Hybrid Key Exchange Works

The hybrid key exchange combines two independent key exchange protocols:

1. **X25519** — Classical elliptic curve Diffie-Hellman (256-bit security against classical attacks)
2. **ML-KEM-768** — Module-Lattice Key Encapsulation Mechanism (NIST PQC standard, 192-bit quantum security)

Both produce independent shared secrets that are concatenated and hashed to produce the final shared secret. An attacker must break **both** algorithms to compromise the key.

### Basic Key Exchange

```fusion
use std::crypto;

fn main() -> int {
    // Generate a hybrid key pair
    let keypair: crypto::HybridKeyPair = crypto::generate_keypair();

    // Extract public key for sharing
    let public_key: bytes = keypair.public_key();
    println("Public key length: %d bytes", public_key.len());

    // Simulate exchange with another party
    let other_keypair: crypto::HybridKeyPair = crypto::generate_keypair();
    let other_public: bytes = other_keypair.public_key();

    // Derive shared secret
    let shared_secret: bytes = keypair.derive_shared_secret(other_public);
    println("Shared secret length: %d bytes", shared_secret.len());

    // Use the shared secret for encryption
    let plaintext: string = "Secret message";
    let ciphertext: bytes = crypto::encrypt(shared_secret, plaintext);
    println("Encrypted: %d bytes", ciphertext.len());

    return 0;
}
```

### Key Exchange Protocol

```fusion
use std::crypto;

struct SecureChannel {
    local_keypair: crypto::HybridKeyPair,
    shared_secret: Option<bytes>,
}

impl SecureChannel {
    fn new() -> SecureChannel {
        return SecureChannel {
            local_keypair: crypto::generate_keypair(),
            shared_secret: None,
        };
    }

    fn get_public_key(self) -> bytes {
        return self.local_keypair.public_key();
    }

    fn complete_exchange(mut self, peer_public: bytes) -> bytes {
        self.shared_secret = Some(self.local_keypair.derive_shared_secret(peer_public));
        return self.shared_secret.unwrap();
    }

    fn encrypt(self, plaintext: string) -> bytes {
        match self.shared_secret {
            Some(secret) => crypto::encrypt(secret, plaintext),
            None => panic!("No shared secret established"),
        }
    }

    fn decrypt(self, ciphertext: bytes) -> string {
        match self.shared_secret {
            Some(secret) => crypto::decrypt(secret, ciphertext),
            None => panic!("No shared secret established"),
        }
    }
}

fn main() -> int {
    // Alice
    let alice: SecureChannel = SecureChannel::new();
    let alice_public: bytes = alice.get_public_key();

    // Bob
    let bob: SecureChannel = SecureChannel::new();
    let bob_public: bytes = bob.get_public_key();

    // Exchange public keys and derive shared secrets
    let alice_secret: bytes = alice.complete_exchange(bob_public);
    let bob_secret: bytes = bob.complete_exchange(alice_public);

    // Both secrets should be identical
    println("Secrets match: %d", alice_secret == bob_secret);

    // Secure communication
    let message: string = "Hello, Bob!";
    let encrypted: bytes = alice.encrypt(message);
    let decrypted: string = bob.decrypt(encrypted);
    println("Decrypted: %s", decrypted);

    return 0;
}
```

### Key Derivation and Session Keys

```fusion
use std::crypto;

fn main() -> int {
    let alice: crypto::HybridKeyPair = crypto::generate_keypair();
    let bob: crypto::HybridKeyPair = crypto::generate_keypair();

    let shared_secret: bytes = alice.derive_shared_secret(bob.public_key());

    // Derive multiple session keys from the shared secret
    let encryption_key: bytes = crypto::hkdf(
        shared_secret,
        salt: "fusion-encryption-v1",
        info: "session-encryption-key",
        length: 32,
    );

    let mac_key: bytes = crypto::hkdf(
        shared_secret,
        salt: "fusion-mac-v1",
        info: "session-mac-key",
        length: 32,
    );

    let iv_key: bytes = crypto::hkdf(
        shared_secret,
        salt: "fusion-iv-v1",
        info: "session-iv-key",
        length: 12,
    );

    println("Encryption key: %d bytes", encryption_key.len());
    println("MAC key: %d bytes", mac_key.len());
    println("IV key: %d bytes", iv_key.len());

    return 0;
}
```

---

## Hybrid Signatures (Ed25519 + ML-DSA-65)

Digital signatures provide authentication and integrity. Fusion combines Ed25519 (classical) with ML-DSA-65 (post-quantum).

### How Hybrid Signatures Work

The hybrid signature combines two independent signature schemes:

1. **Ed25519** — Classical EdDSA signature (fast, compact, 128-bit security)
2. **ML-DSA-65** — Module-Lattice Digital Signature Algorithm (NIST PQC standard, quantum-resistant)

Both signatures are included in the signed output. Verification succeeds only if **both** signatures are valid.

### Creating Signatures

```fusion
use std::crypto;

fn main() -> int {
    // Generate signing key pair
    let signing_key: crypto::HybridSigningKey = crypto::generate_signing_key();
    let verifying_key: crypto::VerifyingKey = signing_key.verifying_key();

    // Message to sign
    let message: string = "Important document content";

    // Create signature
    let signature: bytes = signing_key.sign(message);
    println("Signature length: %d bytes", signature.len());

    // Verify signature
    let valid: bool = verifying_key.verify(message, signature);
    println("Signature valid: %d", valid);

    // Tampered message
    let tampered: string = "Tampered content";
    let invalid: bool = verifying_key.verify(tampered, signature);
    println("Tampered signature valid: %d", invalid);

    return 0;
}
```

### Signed Document Protocol

```fusion
use std::crypto;

struct SignedDocument {
    content: string,
    signature: bytes,
    signer_key: crypto::VerifyingKey,
}

impl SignedDocument {
    fn sign(content: string, signing_key: crypto::HybridSigningKey) -> SignedDocument {
        let signature: bytes = signing_key.sign(content);
        return SignedDocument {
            content,
            signature,
            signer_key: signing_key.verifying_key(),
        };
    }

    fn verify(self) -> bool {
        return self.signer_key.verify(self.content, self.signature);
    }

    fn content(self) -> string {
        return self.content;
    }
}

fn main() -> int {
    // Create a signed document
    let signing_key: crypto::HybridSigningKey = crypto::generate_signing_key();
    let doc: SignedDocument = SignedDocument::sign(
        "Contract terms and conditions",
        signing_key,
    );

    // Verify the document
    let valid: bool = doc.verify();
    println("Document valid: %d", valid);
    println("Content: %s", doc.content());

    return 0;
}
```

### Signature with Timestamp and Metadata

```fusion
use std::crypto;
use std::time;

struct TimestampedSignature {
    message: string,
    signature: bytes,
    signer_key: crypto::VerifyingKey,
    timestamp: int,
    metadata: string,
}

impl TimestampedSignature {
    fn sign(message: string, signing_key: crypto::HybridSigningKey, metadata: string) -> TimestampedSignature {
        // Include timestamp and metadata in the signed payload
        let timestamp: int = time::now();
        let payload: string = "%s|%d|%s" % (message, timestamp, metadata);
        let signature: bytes = signing_key.sign(payload);

        return TimestampedSignature {
            message,
            signature,
            signer_key: signing_key.verifying_key(),
            timestamp,
            metadata,
        };
    }

    fn verify(self) -> bool {
        let payload: string = "%s|%d|%s" % (self.message, self.timestamp, self.metadata);
        return self.signer_key.verify(payload, self.signature);
    }
}

fn main() -> int {
    let signing_key: crypto::HybridSigningKey = crypto::generate_signing_key();

    let signed: TimestampedSignature = TimestampedSignature::sign(
        "Document content here",
        signing_key,
        "version=1.0;author=alice",
    );

    println("Timestamp: %d", signed.timestamp);
    println("Valid: %d", signed.verify());

    return 0;
}
```

---

## 50/50 Enforcement Policy

Fusion enforces a **50/50 policy** — all cryptographic operations must use both classical and post-quantum algorithms.

### How It Works

```fusion
use std::crypto;

// The 50/50 policy is enforced at the type level:
// - HybridKeyPair contains both X25519 and ML-KEM-768 keys
// - HybridSigningKey contains both Ed25519 and ML-DSA-65 keys
// - You cannot use only one algorithm

fn main() -> int {
    // This is the ONLY way to create keys
    let keypair: crypto::HybridKeyPair = crypto::generate_keypair();

    // This would NOT compile:
    // let x25519_only: crypto::X25519KeyPair = crypto::generate_x25519();
    // Error: Must use hybrid key pair

    // The shared secret combines both algorithms
    let shared: bytes = keypair.derive_shared_secret(other_public);

    // Even if one algorithm is broken, the other provides security
    println("Hybrid security: both classical and PQC algorithms active");

    return 0;
}
```

### Policy Violations

```fusion
// These patterns are NOT allowed:

// 1. Using only classical crypto
// crypto::x25519::generate()  // ERROR: Not available

// 2. Using only PQC
// crypto::mlkem::generate()   // ERROR: Not available

// 3. Bypassing the policy
// @unsafe
// fn bypass_policy() { ... }  // WARNING: Crypto bypass detected

// Only the hybrid API is available:
fn main() -> int {
    // Correct: Hybrid key exchange
    let keypair: crypto::HybridKeyPair = crypto::generate_keypair();

    // Correct: Hybrid signatures
    let signing_key: crypto::HybridSigningKey = crypto::generate_signing_key();

    println("Using hybrid cryptography (50/50 policy enforced)");
    return 0;
}
```

### Policy Configuration

```fusion
use std::crypto;

fn main() -> int {
    // The 50/50 policy can be configured at the project level
    let policy_config: crypto::PolicyConfig = crypto::PolicyConfig {
        // Require both classical and PQC for key exchange
        key_exchange_mode: crypto::KeyExchangeMode::Hybrid,

        // Require both classical and PQC for signatures
        signature_mode: crypto::SignatureMode::Hybrid,

        // Minimum key sizes
        min_classical_key_bits: 256,
        min_pqc_key_bits: 192,

        // Algorithm allowlist (empty = use defaults)
        allowed_algorithms: [],
    };

    // Validate policy
    let valid: bool = crypto::validate_policy(policy_config);
    println("Policy valid: %d", valid);

    return 0;
}
```

---

## NeuralSeal PQC

NeuralSeal is Fusion's integrated PQC encryption scheme optimized for machine learning workloads. It provides homomorphic properties that allow computation on encrypted data.

### NeuralSeal Overview

NeuralSeal extends the hybrid approach with:
- **Encrypted inference**: Run ML models on encrypted inputs
- **Encrypted training**: Train models without exposing training data
- **Key rotation**: Rotate keys without re-encrypting data
- **Batched operations**: Process multiple encrypted values efficiently

### Basic NeuralSeal Usage

```fusion
use std::crypto::neuralseal;

fn main() -> int {
    // Generate NeuralSeal keys
    let keys: neuralseal::KeyPair = neuralseal::KeyPair::generate(neuralseal::Config {
        poly_modulus_degree: 8192,
        plain_modulus: 0x10001,  // 65537
        coeff_modulus_sizes: [60, 40, 40, 60],
    });

    // Encrypt a value
    let plaintext: float = 42.0;
    let encrypted: neuralseal::Ciphertext = keys.encrypt(plaintext);
    println("Encrypted value: %d bytes", encrypted.len());

    // Perform computation on encrypted data
    let encrypted_result: neuralseal::Ciphertext = encrypted.add(keys.encrypt(8.0));
    let encrypted_product: neuralseal::Ciphertext = encrypted.mul(keys.encrypt(2.0));

    // Decrypt results
    let result_add: float = keys.decrypt(encrypted_result);
    let result_mul: float = keys.decrypt(encrypted_product);
    println("42 + 8 = %f", result_add);
    println("42 * 2 = %f", result_mul);

    return 0;
}
```

### Encrypted ML Inference

```fusion
use std::crypto::neuralseal;
use std::ml;

fn main() -> int {
    // Generate NeuralSeal keys for encrypted inference
    let keys: neuralseal::KeyPair = neuralseal::KeyPair::generate(neuralseal::Config {
        poly_modulus_degree: 16384,
        plain_modulus: 0x10001,
        coeff_modulus_sizes: [60, 40, 40, 40, 40, 60],
    });

    // Load a pre-trained model
    let model: Network = Network::load("model.bin");

    // Encrypt input data
    let input: ml::Tensor = ml::tensor([1.0, 2.0, 3.0, 4.0]);
    let encrypted_input: neuralseal::EncryptedTensor = keys.encrypt_tensor(input);
    println("Encrypted input: %d bytes", encrypted_input.len());

    // Run inference on encrypted data
    let encrypted_output: neuralseal::EncryptedTensor = model.encrypted_forward(encrypted_input);

    // Decrypt the result
    let output: ml::Tensor = keys.decrypt_tensor(encrypted_output);
    println("Inference result: %s", output.to_string());

    return 0;
}
```

### Key Rotation

```fusion
use std::crypto::neuralseal;

fn main() -> int {
    let keys: neuralseal::KeyPair = neuralseal::KeyPair::generate(neuralseal::Config {
        poly_modulus_degree: 8192,
        plain_modulus: 0x10001,
        coeff_modulus_sizes: [60, 40, 40, 60],
    });

    // Encrypt data with old keys
    let plaintext: float = 100.0;
    let encrypted: neuralseal::Ciphertext = keys.encrypt(plaintext);

    // Generate relinearization keys for key rotation
    let relin_keys: neuralseal::RelinearizationKeys = keys.relin_keys();

    // Rotate to new keys
    let new_keys: neuralseal::KeyPair = neuralseal::KeyPair::generate(neuralseal::Config {
        poly_modulus_degree: 8192,
        plain_modulus: 0x10001,
        coeff_modulus_sizes: [60, 40, 40, 60],
    });

    // Generate galois keys for rotation
    let galois_keys: neuralseal::GaloisKeys = keys.galois_keys();

    // Re-encrypt under new keys (without decrypting!)
    let re_encrypted: neuralseal::Ciphertext = new_keys.re_encrypt(
        encrypted,
        relin_keys,
        galois_keys,
    );

    // Decrypt with new keys
    let result: float = new_keys.decrypt(re_encrypted);
    println("After key rotation: %f", result);

    return 0;
}
```

---

## Secure Transport (PQC TLS)

Fusion provides built-in PQC TLS for secure network communication.

### TLS Client

```fusion
use std::net::tls;

fn main() -> int {
    // Connect with PQC TLS
    let connection: tls::TlsStream = tls::connect(
        "api.example.com",
        443,
        tls::Config {
            verify_certificates: true,
            min_protocol_version: tls::Version::Tls13,
            cipher_suites: tls::CipherSuite::HybridPQC,
        },
    );

    // Send HTTP request
    let request: string = "GET / HTTP/1.1\r\nHost: api.example.com\r\n\r\n";
    connection.write(request);

    // Read response
    let response: string = connection.read();
    println("Response: %s", response);

    connection.close();
    return 0;
}
```

### TLS Server

```fusion
use std::net::tls;

fn main() -> int {
    // Create TLS server
    let server: tls::TlsServer = tls::TlsServer::bind(
        "0.0.0.0",
        8443,
        tls::ServerConfig {
            certificate: tls::load_certificate("server.crt"),
            private_key: tls::load_private_key("server.key"),
            min_protocol_version: tls::Version::Tls13,
            cipher_suites: tls::CipherSuite::HybridPQC,
        },
    );

    println("Server listening on port 8443");

    // Accept connections
    loop {
        let client: tls::TlsStream = server.accept();
        // Handle client in a new fiber
        spawn handle_client(client);
    }

    return 0;
}

fn handle_client(client: tls::TlsStream) {
    let request: string = client.read();
    let response: string = "HTTP/1.1 200 OK\r\n\r\nHello, World!";
    client.write(response);
    client.close();
}
```

### Mutual TLS (mTLS) with PQC

```fusion
use std::net::tls;

fn main() -> int {
    // Client with mutual TLS (both client and server verify certificates)
    let connection: tls::TlsStream = tls::connect_mutual(
        "api.example.com",
        443,
        tls::MutualConfig {
            // Client certificate (signed with hybrid signature)
            certificate: tls::load_certificate("client.crt"),
            private_key: tls::load_private_key("client.key"),

            // Server verification
            verify_certificates: true,
            trusted_ca: tls::load_ca_bundle("ca-bundle.crt"),

            // PQC settings
            min_protocol_version: tls::Version::Tls13,
            cipher_suites: tls::CipherSuite::HybridPQC,
        },
    );

    // Send authenticated request
    let request: string = "GET /secure HTTP/1.1\r\nHost: api.example.com\r\nAuthorization: Bearer token\r\n\r\n";
    connection.write(request);

    let response: string = connection.read();
    println("Secure response: %s", response);

    connection.close();
    return 0;
}
```

### Certificate Pinning

```fusion
use std::net::tls;

fn main() -> int {
    // Pin specific certificate or public key
    let pinned_cert: bytes = std::fs::read("pinned-cert.der");

    let connection: tls::TlsStream = tls::connect_pinned(
        "api.example.com",
        443,
        tls::PinnedConfig {
            pinned_certificate: pinned_cert,
            pinning_type: tls::PinType::CertificateHash,
            min_protocol_version: tls::Version::Tls13,
            cipher_suites: tls::CipherSuite::HybridPQC,
        },
    );

    let request: string = "GET / HTTP/1.1\r\nHost: api.example.com\r\n\r\n";
    connection.write(request);

    let response: string = connection.read();
    println("Response: %s", response);

    connection.close();
    return 0;
}
```

---

## Configuration in Fusion.toml

```toml
[crypto]
# Enforce hybrid cryptography (50/50 policy)
enforce_hybrid = true

# Disable classical-only operations
allow_classical_only = false

# Disable PQC-only operations
allow_pqc_only = false

[crypto.key_exchange]
# Key exchange algorithms
classical_algorithm = "X25519"
pqc_algorithm = "ML-KEM-768"

# Key derivation function
kdf = "HKDF-SHA256"
kdf_info_prefix = "fusion-key-exchange"

# Key lifetime (seconds)
key_lifetime = 3600

[crypto.signatures]
# Signature algorithms
classical_algorithm = "Ed25519"
pqc_algorithm = "ML-DSA-65"

# Signature lifetime (seconds)
signature_lifetime = 86400

[crypto.neuralseal]
# NeuralSeal configuration
enabled = true
poly_modulus_degree = 8192
plain_modulus = 65537
coeff_modulus_sizes = [60, 40, 40, 60]

# Security level
security_level = 128

# Enable key rotation
key_rotation = true
key_rotation_interval = 7200

[crypto.tls]
# TLS configuration
min_version = "TLS1.3"
cipher_suites = ["HybridPQC"]

# Certificate verification
verify_certificates = true
verify_hostnames = true

# Client certificate (for mTLS)
client_certificate = ""
client_private_key = ""

# Trusted CA bundle
ca_bundle = ""

[crypto.storage]
# Key storage
storage_backend = "file"  # Options: "file", "hsm", "kms"
key_file_path = "./keys/"
encrypt_keys_at_rest = true

# HSM settings (if using HSM backend)
hsm_provider = "pkcs11"
hsm_library = "/usr/lib/pkcs11.so"
hsm_slot = 0
```

---

## API Reference

### Key Exchange

| Function | Description |
|----------|-------------|
| `crypto::generate_keypair()` | Generate hybrid key pair |
| `keypair.public_key()` | Get public key bytes |
| `keypair.derive_shared_secret(peer_public)` | Derive shared secret |

### Signatures

| Function | Description |
|----------|-------------|
| `crypto::generate_signing_key()` | Generate hybrid signing key |
| `signing_key.verifying_key()` | Get verifying key |
| `signing_key.sign(message)` | Sign a message |
| `verifying_key.verify(message, signature)` | Verify signature |

### Encryption

| Function | Description |
|----------|-------------|
| `crypto::encrypt(key, plaintext)` | Encrypt with shared secret |
| `crypto::decrypt(key, ciphertext)` | Decrypt with shared secret |

### NeuralSeal

| Function | Description |
|----------|-------------|
| `neuralseal::KeyPair::generate(config)` | Generate NeuralSeal keys |
| `keys.encrypt(plaintext)` | Encrypt a value |
| `keys.decrypt(ciphertext)` | Decrypt a value |
| `keys.encrypt_tensor(tensor)` | Encrypt a tensor |
| `keys.decrypt_tensor(enc_tensor)` | Decrypt a tensor |
| `keys.relin_keys()` | Get relinearization keys |
| `keys.galois_keys()` | Get Galois keys |

### TLS

| Function | Description |
|----------|-------------|
| `tls::connect(host, port, config)` | Connect to TLS server |
| `tls::connect_mutual(host, port, config)` | Connect with mTLS |
| `tls::connect_pinned(host, port, config)` | Connect with certificate pinning |
| `tls::TlsServer::bind(addr, port, config)` | Create TLS server |
| `stream.write(data)` | Write to TLS stream |
| `stream.read()` | Read from TLS stream |

---

## Complete Examples

### Secure Communication

```fusion
use std::crypto;

fn main() -> int {
    // Alice and Bob establish a secure channel
    let alice: crypto::HybridKeyPair = crypto::generate_keypair();
    let bob: crypto::HybridKeyPair = crypto::generate_keypair();

    // Key exchange
    let shared_secret: bytes = alice.derive_shared_secret(bob.public_key());

    // Derive session keys
    let enc_key: bytes = crypto::hkdf(shared_secret, salt: "enc", info: "session", length: 32);
    let mac_key: bytes = crypto::hkdf(shared_secret, salt: "mac", info: "session", length: 32);

    // Alice sends an encrypted message
    let message: string = "Hello Bob! This is a secret message.";
    let nonce: bytes = crypto::random_bytes(12);
    let ciphertext: bytes = crypto::aes_gcm_encrypt(enc_key, nonce, message);
    let tag: bytes = ciphertext[-16..];  // Authentication tag

    // Alice signs the message
    let alice_signing: crypto::HybridSigningKey = crypto::generate_signing_key();
    let signature: bytes = alice_signing.sign(ciphertext);

    // Send to Bob: ciphertext + nonce + tag + signature + Alice's public verifying key
    let alice_verify_key: bytes = alice_signing.verifying_key().to_bytes();

    // Bob receives and verifies
    // 1. Verify signature
    let alice_verify: crypto::VerifyingKey = crypto::VerifyingKey::from_bytes(alice_verify_key);
    let sig_valid: bool = alice_verify.verify(ciphertext, signature);
    println("Signature valid: %d", sig_valid);

    if !sig_valid {
        println("ERROR: Signature verification failed!");
        return 1;
    }

    // 2. Decrypt message
    let decrypted: string = crypto::aes_gcm_decrypt(enc_key, nonce, ciphertext);
    println("Decrypted: %s", decrypted);

    // 3. Bob replies
    let reply: string = "Hi Alice! Got your message.";
    let reply_nonce: bytes = crypto::random_bytes(12);
    let reply_ciphertext: bytes = crypto::aes_gcm_encrypt(enc_key, reply_nonce, reply);

    let bob_signing: crypto::HybridSigningKey = crypto::generate_signing_key();
    let reply_sig: bytes = bob_signing.sign(reply_ciphertext);

    // Verify and decrypt reply
    let reply_decrypted: string = crypto::aes_gcm_decrypt(enc_key, reply_nonce, reply_ciphertext);
    println("Bob's reply: %s", reply_decrypted);

    return 0;
}
```

### Digital Signatures (File Signing)

```fusion
use std::crypto;
use std::fs;

fn main() -> int {
    // Generate signing key
    let signing_key: crypto::HybridSigningKey = crypto::generate_signing_key();
    let verifying_key: crypto::VerifyingKey = signing_key.verifying_key();

    // Save the verifying key for later verification
    fs::write("public_key.bin", verifying_key.to_bytes());

    // Read file to sign
    let file_content: bytes = fs::read("document.pdf");

    // Create signature
    let signature: bytes = signing_key.sign(file_content);

    // Save signature
    fs::write("document.pdf.sig", signature);

    println("File signed successfully");
    println("Signature length: %d bytes", signature.len());

    // Later: verify the signature
    let stored_verify_key: bytes = fs::read("public_key.bin");
    let verify_key: crypto::VerifyingKey = crypto::VerifyingKey::from_bytes(stored_verify_key);

    let stored_content: bytes = fs::read("document.pdf");
    let stored_sig: bytes = fs::read("document.pdf.sig");

    let valid: bool = verify_key.verify(stored_content, stored_sig);
    println("Signature valid: %d", valid);

    // Verify against original hash
    let file_hash: bytes = crypto::sha256(stored_content);
    println("File hash: %s", file_hash.to_hex());

    return 0;
}
```

### Secure File Transfer

```fusion
use std::crypto;
use std::net::tls;
use std::fs;

fn main() -> int {
    // Client: encrypt and send a file
    let keypair: crypto::HybridKeyPair = crypto::generate_keypair();

    // Read file
    let file_content: bytes = fs::read("sensitive-data.bin");
    println("File size: %d bytes", file_content.len());

    // Generate random file encryption key
    let file_key: bytes = crypto::random_bytes(32);
    let file_nonce: bytes = crypto::random_bytes(12);

    // Encrypt file content
    let encrypted_file: bytes = crypto::aes_gcm_encrypt(file_key, file_nonce, file_content);

    // Encrypt the file key with recipient's public key
    let recipient_keypair: crypto::HybridKeyPair = crypto::generate_keypair();
    let encrypted_key: bytes = crypto::hybrid_encrypt(
        recipient_keypair.public_key(),
        file_key,
    );

    // Connect to server and send
    let connection: tls::TlsStream = tls::connect(
        "file-server.example.com",
        8443,
        tls::Config {
            verify_certificates: true,
            min_protocol_version: tls::Version::Tls13,
            cipher_suites: tls::CipherSuite::HybridPQC,
        },
    );

    // Send encrypted key, nonce, and encrypted file
    connection.write(encrypted_key);
    connection.write(file_nonce);
    connection.write(encrypted_file);

    // Sign the transfer
    let signing_key: crypto::HybridSigningKey = crypto::generate_signing_key();
    let transfer_hash: bytes = crypto::sha256(encrypted_file);
    let transfer_sig: bytes = signing_key.sign(transfer_hash);
    connection.write(transfer_sig);

    println("File transferred securely");

    connection.close();
    return 0;
}
```

---

## Tips and Best Practices

1. **Always use hybrid crypto**: Never try to use only classical or only PQC algorithms.
2. **Verify certificates**: Always enable certificate verification in production.
3. **Use TLS 1.3**: It provides the best security and performance.
4. **Rotate keys regularly**: Generate new key pairs periodically.
5. **Secure key storage**: Use hardware security modules (HSMs) for production keys.
6. **Use NeuralSeal for ML**: When processing sensitive data with ML models, use NeuralSeal encryption.
7. **Enable key rotation**: Configure automatic key rotation for long-running services.
8. **Monitor algorithm status**: Keep up with NIST PQC standardization updates.
9. **Test with both algorithms**: Ensure your system works even if one algorithm is weakened.

---

## Cross-References

- **Chapter 1**: Getting Started for installation
- **Chapter 8**: Quantum Computing for quantum algorithms
- **Chapter 9**: Machine Learning for encrypted ML inference
- **Chapter 10**: Concurrency for secure network servers
- **Chapter 14**: Examples for complete PQC chat application
