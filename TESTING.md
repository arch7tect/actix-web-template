# Testing Strategy

This document outlines the testing strategy for the Actix Web memos application.

## Test Structure

The test suite is organized into several categories:

```
tests/
├── common/
│   ├── mod.rs          # Test setup and helpers
│   └── fixtures.rs     # Test data fixtures
├── api_tests.rs        # REST API integration tests (10 tests)
├── service_tests.rs    # Service layer unit tests (12 tests)
├── repository_tests.rs # Repository layer unit tests (11 tests)
└── web_tests.rs        # HTML/web endpoint integration tests (9 tests)
```

## Test Categories

### 1. Unit Tests (5 tests in src/lib.rs)
- **Location**: `src/utils/sanitize.rs` (inline tests)
- **Purpose**: Test HTML sanitization utilities
- **Coverage**: Input sanitization, XSS prevention

### 2. Service Layer Tests (12 tests)
- **File**: `tests/service_tests.rs`
- **Purpose**: Test business logic layer
- **Coverage**:
  - CRUD operations (create, read, update, delete)
  - Partial updates (patch)
  - Toggle completion status
  - Pagination
  - Filtering by completion status
  - Validation errors

### 3. Repository Tests (11 tests)
- **File**: `tests/repository_tests.rs`
- **Purpose**: Test database access layer
- **Coverage**:
  - Database CRUD operations
  - Pagination logic
  - Filtering (by completion status)
  - Sorting (by different fields, asc/desc)
  - Edge cases (not found, invalid IDs)

### 4. REST API Integration Tests (10 tests)
- **File**: `tests/api_tests.rs`
- **Purpose**: Test REST API endpoints end-to-end
- **Coverage**:
  - `POST /api/v1/memos` - Create memo
  - `GET /api/v1/memos` - List memos with pagination
  - `GET /api/v1/memos/{id}` - Get single memo
  - `PUT /api/v1/memos/{id}` - Update memo
  - `PATCH /api/v1/memos/{id}` - Partial update
  - `DELETE /api/v1/memos/{id}` - Delete memo
  - `PATCH /api/v1/memos/{id}/complete` - Toggle completion
  - Error scenarios (404, validation errors)

### 5. Web/HTML Integration Tests (9 tests)
- **File**: `tests/web_tests.rs`
- **Purpose**: Test server-side rendered HTML endpoints
- **Coverage**:
  - `GET /` - Index page
  - `GET /web/memos` - Memos list component
  - `GET /web/memos/new` - New memo form
  - `POST /web/memos` - Create memo (form submission)
  - `GET /web/memos/{id}/edit` - Edit form
  - `PUT /web/memos/{id}` - Update memo (form submission)
  - `DELETE /web/memos/{id}` - Delete memo
  - `PATCH /web/memos/{id}/toggle` - Toggle completion
  - Validation errors

## Test Helpers

### Common Module (`tests/common/`)

**`setup_test_db()`**: Creates a database connection for tests
**`setup_test_state()`**: Creates complete AppState for integration tests
**`create_test_memo_dto()`**: Fixture for creating test memo DTOs

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Suite
```bash
cargo test --test api_tests
cargo test --test service_tests
cargo test --test repository_tests
cargo test --test web_tests
```

### Run Specific Test
```bash
cargo test test_create_memo
```

### Run Tests with Output
```bash
cargo test -- --nocaptured
```

## Test Database

Tests use the same database URL configured in `.env`, but each test:
- Creates its own test data
- Cleans up after itself (deletes created records)
- Runs in isolation

**Important**: Ensure PostgreSQL is running before running tests:
```bash
docker-compose up -d postgres
```

## Code Quality

### Linting
```bash
cargo clippy -- -D warnings
```

### Formatting
```bash
cargo fmt
cargo fmt --check
```

## Test Coverage

**Current Test Count**: 47 total tests
- Unit tests: 5
- Service tests: 12
- Repository tests: 11
- API integration tests: 10
- Web integration tests: 9

**Coverage Target**: >70% code coverage

### Measuring Coverage (Optional)

Install tarpaulin:
```bash
cargo install cargo-tarpaulin
```

Generate coverage report:
```bash
cargo tarpaulin --out Html --output-dir coverage
```

View report:
```bash
open coverage/index.html
```

## Continuous Integration

Before committing:
1. Run all tests: `cargo test`
2. Run linter: `cargo clippy -- -D warnings`
3. Format code: `cargo fmt`
4. Ensure no compilation warnings

## Testing Best Practices

1. **Test Isolation**: Each test creates and cleans up its own data
2. **Descriptive Names**: Test names clearly describe what is being tested
3. **Arrange-Act-Assert**: Tests follow AAA pattern
4. **No Test Dependencies**: Tests can run in any order
5. **Async Tests**: All tests use `#[tokio::test]` for async support
6. **Cleanup**: Tests always clean up created resources
7. **Realistic Data**: Use meaningful test data, not just "test"
8. **Edge Cases**: Test both happy paths and error scenarios

## Future Enhancements

- Add property-based testing with `proptest`
- Add benchmarks with `criterion`
- Add mutation testing
- Increase coverage to >80%
- Add performance regression tests