# Contributing to WEEX Rust SDK

Thank you for your interest in contributing! This document provides guidelines for contributing to the WEEX Rust SDK.

## 🌟 Ways to Contribute

- **Bug Reports**: Found a bug? Open an issue with reproduction steps
- **Feature Requests**: Have an idea? Share it in the issues
- **Code Contributions**: Submit a pull request
- **Documentation**: Help improve our docs
- **Examples**: Add usage examples

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ (stable)
- Git

### Development Setup

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/weex-rust-sdk.git
cd weex-rust-sdk

# Build
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy
```

## 📝 Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Write tests** for your changes
4. **Ensure** all tests pass (`cargo test`)
5. **Format** your code (`cargo fmt`)
6. **Lint** your code (`cargo clippy`)
7. **Commit** your changes
8. **Push** and open a Pull Request

### Commit Message Format

```
type(scope): description

[optional body]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## 📐 Code Style

- Follow Rust idioms and conventions
- Use `rustfmt` for formatting
- Address all `clippy` warnings
- Document public APIs with doc comments

```rust
/// Calculate position size based on risk parameters.
/// 
/// # Arguments
/// 
/// * `balance` - Available balance in USDT
/// * `risk_pct` - Risk percentage (0.0 - 1.0)
/// 
/// # Returns
/// 
/// Position size in base currency
/// 
/// # Example
/// 
/// ```
/// let size = calculate_position_size(1000.0, 0.02);
/// assert!(size <= 20.0);
/// ```
pub fn calculate_position_size(balance: f64, risk_pct: f64) -> f64 {
    balance * risk_pct
}
```

## 🔒 Security

- Never commit API keys or secrets
- Report security issues privately
- Use secure coding practices

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing! 🎉
