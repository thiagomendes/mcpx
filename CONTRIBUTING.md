# Contributing to mcpx

Thank you for your interest in contributing to mcpx. This document provides guidelines and instructions for contributing.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Coding Standards](#coding-standards)
5. [Commit Guidelines](#commit-guidelines)
6. [Pull Request Process](#pull-request-process)
7. [Testing](#testing)
8. [Documentation](#documentation)

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to mail.thiagomendes@gmail.com.

## Getting Started

### Fork the Repository

1. Fork the repository on GitHub
2. Clone your fork locally:

```bash
git clone https://github.com/YOUR_USERNAME/mcpx.git
cd mcpx
```

3. Add the upstream repository as a remote:

```bash
git remote add upstream https://github.com/thiagomendes/mcpx.git
```

4. Keep your fork synchronized:

```bash
git fetch upstream
git checkout main
git merge upstream/main
```

### Set Up Development Environment

Follow the [Quick Start](README.md#quick-start) section in the README to set up your local development environment using Docker Compose.

## Development Workflow

### Create a Feature Branch

Always create a new branch for your work:

```bash
git checkout main
git pull upstream main
git checkout -b feature/your-feature-name
```

Use descriptive branch names:

| Type | Format | Example |
|------|--------|---------|
| Feature | `feature/description` | `feature/add-rate-limiting` |
| Bug fix | `fix/description` | `fix/oauth-redirect-loop` |
| Documentation | `docs/description` | `docs/update-api-reference` |
| Refactor | `refactor/description` | `refactor/metrics-service` |

### Make Your Changes

1. Write code following the [Coding Standards](#coding-standards)
2. Add or update tests as needed
3. Update documentation if applicable
4. Ensure all tests pass

### Keep Commits Atomic

Each commit should represent a single logical change. This makes code review easier and helps with debugging.

## Coding Standards

Refer to [AGENT.md](AGENT.md) for AI-assisted development guidelines. Key points:

### Backend (Rust)

| Requirement | Description |
|-------------|-------------|
| Linting | Run `cargo clippy` before committing |
| Testing | Run `cargo test` to verify changes |
| Error messages | Centralize in `messages.rs` |
| SQL queries | Define as constants at file top |
| Dead code | Use `#[allow(dead_code)]` when necessary |

### Frontend (Vue + TypeScript)

| Requirement | Description |
|-------------|-------------|
| Linting | Run `npm run lint` before committing |
| Testing | Run `npm test` to verify changes |
| Constants | Centralize in `constants.ts` |
| Types | Avoid `any`, use `unknown` or specific types |
| Unused variables | Prefix with underscore (`_unused`) |

## Commit Guidelines

Follow the Conventional Commits specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Formatting, no code change |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `test` | Adding or updating tests |
| `chore` | Maintenance tasks |

### Examples

```
feat(governance): add tool prefix configuration

fix(proxy): resolve session timeout on stateful servers

docs(readme): update deployment instructions

refactor(metrics): extract query builder to separate module
```

## Pull Request Process

### Before Submitting

1. Ensure your branch is up to date with main:

```bash
git fetch upstream
git rebase upstream/main
```

2. Run all validation commands:

```bash
# Backend
cd backend
cargo clippy
cargo test

# Frontend
cd frontend
npm run lint
npm test
```

3. Verify the application works:

```bash
docker compose build && docker compose up -d
```

### Submitting a Pull Request

1. Push your branch to your fork:

```bash
git push origin feature/your-feature-name
```

2. Open a Pull Request against `main` in the upstream repository

3. Fill out the PR template with:
   - Description of changes
   - Related issue numbers
   - Testing performed
   - Screenshots (if UI changes)

### Review Process

1. Maintainers will review your PR
2. Address any requested changes
3. Once approved, a maintainer will merge

### After Merge

Delete your feature branch:

```bash
git checkout main
git pull upstream main
git branch -d feature/your-feature-name
git push origin --delete feature/your-feature-name
```

## Testing

### Backend Tests

```bash
cd backend
cargo test
```

### Frontend Tests

```bash
cd frontend
npm test
```

### Integration Testing

Start the application and verify functionality manually:

```bash
docker compose up -d
curl http://localhost:8080/api/health
```

## Documentation

When contributing documentation:

1. Use clear, concise language
2. Include code examples where appropriate
3. Keep formatting consistent with existing docs
4. Update the table of contents if adding sections

## Questions

If you have questions about contributing, open an issue or contact mail.thiagomendes@gmail.com.

Thank you for contributing to mcpx.
