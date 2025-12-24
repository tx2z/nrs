# Contributing to nrs

Thank you for your interest in contributing to nrs! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git

### Getting Started

```bash
# Clone the repository
git clone https://github.com/user/nrs
cd nrs

# Build the project
cargo build

# Run tests
cargo test

# Run the project
cargo run

# Run with arguments
cargo run -- --list
cargo run -- --debug
```

### Project Structure

```
nrs/
├── src/
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Library root
│   ├── cli.rs           # CLI argument parsing
│   ├── error.rs         # Error types
│   ├── config/          # Configuration system
│   ├── package/         # Package.json parsing
│   ├── history/         # History tracking
│   ├── tui/             # Terminal UI
│   ├── runner/          # Script execution
│   ├── filter/          # Fuzzy filtering
│   └── utils/           # Utilities
├── tests/
│   ├── integration/     # Integration tests
│   └── package_parsing.rs
├── docs/                # Documentation
└── benches/             # Benchmarks
```

## Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run integration tests only
cargo test --test integration_tests

# Update snapshot tests
cargo insta test --accept
```

## Code Style

### Formatting

We use `rustfmt` for code formatting:

```bash
# Format all code
cargo fmt

# Check formatting without making changes
cargo fmt --check
```

### Linting

We use `clippy` for linting:

```bash
# Run clippy
cargo clippy

# Run clippy with warnings as errors
cargo clippy -- -D warnings
```

### Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Write doc comments for all public items
- Keep functions small and focused
- Use meaningful variable names
- Add tests for new functionality
- Handle errors gracefully with helpful messages

## Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Test changes
- `chore`: Build/tooling changes

Examples:
```
feat(tui): add multi-select mode
fix(parser): handle empty scripts object
docs(readme): add installation instructions
```

## Pull Request Process

1. **Fork the repository** and create a new branch:
   ```bash
   git checkout -b feat/my-feature
   ```

2. **Make your changes** and ensure:
   - All tests pass: `cargo test`
   - Code is formatted: `cargo fmt`
   - No clippy warnings: `cargo clippy -- -D warnings`
   - Documentation builds: `cargo doc --no-deps`

3. **Write tests** for new functionality

4. **Update documentation** if needed:
   - README.md for user-facing changes
   - Doc comments for API changes
   - CHANGELOG.md for notable changes

5. **Submit a pull request** with:
   - Clear title describing the change
   - Description of what and why
   - Any breaking changes noted

## Reporting Issues

When reporting issues, please include:

- nrs version (`nrs --version`)
- Operating system and version
- Rust version (`rustc --version`)
- Steps to reproduce
- Expected behavior
- Actual behavior
- Any error messages

## Feature Requests

We welcome feature requests! Please:

1. Check existing issues to avoid duplicates
2. Describe the use case
3. Explain why this feature would be useful
4. Consider if it fits the project scope

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Provide constructive feedback
- Focus on the code, not the person

## Getting Help

- Open an issue for bugs or questions
- Check existing documentation and issues first
- Be patient - maintainers are volunteers

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
