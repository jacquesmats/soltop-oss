# Contributing to soltop

Thank you for your interest in contributing to soltop! This document provides guidelines and instructions for contributing.

## Code of Conduct

This project adheres to a code of conduct (see [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)). By participating, you are expected to uphold this code.

## Getting Started

### Development Setup

1. **Install Rust** (1.75 or later):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone the repository**:
   ```bash
   git clone https://github.com/jacquesmats/soltop-oss.git
   cd soltop-oss
   ```

3. **Build and test**:
   ```bash
   cargo build
   cargo test
   cargo run
   ```

### Project Structure

```
soltop/
├── src/
│   ├── rpc/      # Solana RPC client and data fetching
│   ├── stats/    # Statistics collection and aggregation
│   └── ui/       # Terminal interface (ratatui-based)
├── tests/        # Integration tests
└── Cargo.toml
```

## How to Contribute

### Reporting Bugs

Before creating a bug report:
- Check [existing issues](https://github.com/jacquesmats/soltop-oss/issues) to avoid duplicates
- Collect information: Rust version, OS, RPC endpoint used
- Try to reproduce with minimal steps

**Bug Report Template:**
```
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce:
1. Run soltop with '...'
2. Press '...'
3. See error

**Expected behavior**
What you expected to happen.

**Environment:**
 - OS: [e.g., Ubuntu 22.04]
 - Rust version: [e.g., 1.75]
 - soltop version: [e.g., 0.1.0]
 - RPC endpoint: [e.g., mainnet-beta public]
```

### Suggesting Features

Feature requests are welcome! Please:
- Check if the feature is already requested
- Explain the use case clearly
- Consider implementation complexity
- Provide examples if applicable

### Pull Requests

1. **Fork and create a branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**:
   - Write clear, commented code
   - Follow existing code style
   - Add tests for new functionality
   - Update documentation

3. **Test your changes**:
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt
   ```

4. **Commit with clear messages**:
   ```bash
   git commit -m "feat: add X feature"
   ```

   Use conventional commit format:
   - `feat:` New feature
   - `fix:` Bug fix
   - `docs:` Documentation changes
   - `refactor:` Code refactoring
   - `test:` Test changes
   - `chore:` Maintenance tasks
   - `perf:` Performance improvements

5. **Push and create PR**:
   ```bash
   git push origin feature/your-feature-name
   ```

   Then open a pull request on GitHub with:
   - Clear description of changes
   - Reference to related issues
   - Screenshots/examples if applicable

### Code Style

- Run `cargo fmt` before committing (enforces Rust formatting standards)
- Run `cargo clippy` and address warnings
- Follow Rust API guidelines
- Write meaningful comments for complex logic
- Keep functions focused and testable
- Prefer readability over cleverness

### Testing

- Add unit tests in the same file with `#[cfg(test)]`
- Test edge cases and error conditions
- Integration tests go in `tests/` directory
- Aim for meaningful test coverage
- Run `cargo test` before submitting PRs

### Documentation

- Update README.md for user-facing changes
- Add rustdoc comments (`///`) for public APIs
- Update CHANGELOG.md following [Keep a Changelog](https://keepachangelog.com/) format
- Include examples in doc comments where helpful

## Development Workflow

### Running Locally

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run with custom RPC endpoint
cargo run -- --rpc-url https://your-endpoint.com

# Run with verbose output
cargo run -- --verbose

# Build optimized binary
cargo build --release
```

### Debugging

```bash
# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Check for common mistakes
cargo clippy

# Generate documentation
cargo doc --open
```

### Before Submitting a PR

Checklist:
- [ ] Code builds without errors
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (if applicable)
- [ ] Commit messages follow conventional commits format

## Getting Help

- **Questions**: Open a [GitHub Discussion](https://github.com/jacquesmats/soltop-oss/discussions)
- **Bugs**: Open a [GitHub Issue](https://github.com/jacquesmats/soltop-oss/issues)
- **Chat**: Join our community (Discord link coming soon)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Recognition

Contributors will be acknowledged in:
- Release notes
- README.md (for significant contributions)
- Git commit history

Thank you for contributing to soltop!
