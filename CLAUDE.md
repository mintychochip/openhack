# CLAUDE.md — Coding Conventions for AI Assistants

## Core Rule: Extremely Descriptive Native Documentation

Every function and class must use its **native language's documentation format** and be written so that an AI agent can fully understand its behavior without reading the implementation. Documentation must be self-contained and exhaustive.

### Three Mandatory Sections

Every function/class doc must include these three sections:

1. **Expected Behavior** — What it does, all input variations, return value shape and constraints, edge case behavior, type constraints, state transitions, and invariants maintained.
2. **Raises** — Every possible exception or error with the exact conditions that trigger each one, including chained exceptions.
3. **Side Effects** — Every mutation, I/O operation, network call, file write, global state change, cache invalidation, event emission, and logging — with timing (before/after/during main logic) and failure semantics (what happens to side effects if the function throws partway through).

---

## Descriptiveness Requirements

- **No single-sentence descriptions.** Every function doc must answer: What? How? When? What if? What else?
- **Edge cases and boundary conditions** must be documented, not just the happy path.
- **Side effects** must specify timing (before/after/during main logic) and failure semantics (what happens to side effects if the function throws partway through).
- **Raises** must include trigger conditions, not just exception names.
- **Return types** must describe shape and constraints, not just type names.

---

## Scope

- **Public functions/classes** — always documented.
- **Private/internal functions** — documented only if they have side effects, non-trivial logic, or can raise exceptions.
- **Trivial getters/setters** — exempt unless they have side effects.

---

## Language-Specific Templates and Examples

### Python — Google-Style Docstrings

```python
def transfer_funds(source: str, target: str, amount: Decimal) -> Receipt:
    """Transfer funds between accounts with transactional guarantees.

    Expected Behavior:
        Deducts `amount` from `source` account and credits `amount` to
        `target` account atomically. Returns a Receipt with the
        transaction ID, timestamp, and balances of both accounts
        post-transfer. If `source` balance is insufficient, no debit
        occurs and InsufficientFundsError is raised. Transfer is
        idempotent for the same idempotency_key within 24h — a
        repeated request returns the original Receipt.

    Args:
        source: Account ID to debit. Must be a valid, active account.
            Raises AccountNotFoundError if account does not exist
            or is closed.
        target: Account ID to credit. Must be a valid, active account.
            Raises AccountNotFoundError if account does not exist
            or is closed.
        amount: Non-negative Decimal to transfer. Must have at most
            2 decimal places. Raises ValueError if negative or
            exceeds precision.

    Returns:
        Receipt: Contains transaction_id (str), timestamp (UTC datetime),
        source_balance (Decimal), target_balance (Decimal).

    Raises:
        InsufficientFundsError: If source balance < amount after
            pending holds are applied. `available_balance` attribute
            on the exception contains the actual available amount.
        AccountNotFoundError: If source or target account does not
            exist or is closed. `account_id` attribute on the
            exception identifies which account.
        ValueError: If amount is negative or has >2 decimal places.
        DatabaseUnavailableError: If the transaction database is
            unreachable. No partial debit/credit occurs.

    Side Effects:
        - Writes a transaction record to the `transactions` table
          (occurs after both debit and credit succeed).
        - Publishes a `funds.transferred` event to the message queue
          (occurs after transaction record is committed).
        - Increments `transfer_count` metric on the source account.
        - Logs at INFO level with source, target, and amount on
          success; logs at WARNING on insufficient funds.
    """
```

### TypeScript/JavaScript — TSDoc

```typescript
/**
 * Transfer funds between accounts with transactional guarantees.
 *
 * @expectedBehavior Deducts `amount` from `source` and credits `amount`
 * to `target` atomically. Returns a Receipt with transaction ID,
 * timestamp, and post-transfer balances. If source balance is
 * insufficient, no debit occurs and InsufficientFundsError is thrown.
 * Idempotent for the same idempotencyKey within 24h — repeated
 * requests return the original Receipt.
 *
 * @param source - Account ID to debit. Must be a valid, active account.
 * Throws AccountNotFoundError if nonexistent or closed.
 * @param target - Account ID to credit. Must be a valid, active account.
 * @param amount - Non-negative number to transfer. Must be > 0.
 * Throws ValueError if <= 0 or exceeds precision.
 * @returns Receipt containing transactionId, timestamp (UTC),
 * sourceBalance, and targetBalance.
 *
 * @throws {InsufficientFundsError} If source balance < amount after
 * pending holds. Exception has `availableBalance` property.
 * @throws {AccountNotFoundError} If source or target is invalid/closed.
 * Exception has `accountId` property identifying which account.
 * @throws {DatabaseUnavailableError} If the DB is unreachable. No
 * partial state change occurs.
 *
 * @sideeffect Writes a transaction record to the DB after both
 * debit and credit succeed.
 * @sideeffect Publishes `funds.transferred` event to message queue
 * after transaction commit.
 * @sideeffect Increments transferCount metric on source account.
 * @sideeffect Logs at INFO on success, WARNING on insufficient funds.
 */
function transferFunds(source: string, target: string, amount: number): Receipt
```

### Go — GoDoc + Custom Sections

```go
// TransferFunds transfers funds between accounts with transactional guarantees.
//
// # Expected Behavior
//
// Deducts amount from source account and credits amount to target account
// atomically. Returns a Receipt with the transaction ID, timestamp, and
// balances of both accounts post-transfer. If source balance is insufficient,
// no debit occurs and InsufficientFundsError is returned. Transfer is
// idempotent for the same idempotencyKey within 24h — a repeated request
// returns the original Receipt.
//
// # Parameters
//
//	source - Account ID to debit. Must be a valid, active account.
//	    Returns AccountNotFoundError if account does not exist or is closed.
//	target - Account ID to credit. Must be a valid, active account.
//	    Returns AccountNotFoundError if account does not exist or is closed.
//	amount - Non-negative amount to transfer. Must be > 0 and have at most
//	    2 decimal places. Returns ValueError if invalid.
//
// # Returns
//
// Receipt containing transactionId, timestamp (UTC), sourceBalance,
// and targetBalance.
//
// # Errors
//
// InsufficientFundsError: If source balance < amount after pending holds.
// The availableBalance field on the error contains the actual available amount.
// AccountNotFoundError: If source or target account does not exist or is closed.
// The AccountID field identifies which account.
// ValueError: If amount is negative or has >2 decimal places.
// DatabaseUnavailableError: If the transaction database is unreachable.
// No partial debit/credit occurs.
//
// # Side Effects
//
// - Writes a transaction record to the transactions table (after both
//   debit and credit succeed).
// - Publishes a funds.transferred event to the message queue (after
//   transaction record is committed).
// - Increments transferCount metric on the source account.
// - Logs at INFO on success, WARNING on insufficient funds.
func TransferFunds(source, target string, amount decimal.Decimal) (Receipt, error)
```

### Rust — Doc Comments

```rust
/// Transfer funds between accounts with transactional guarantees.
///
/// # Expected Behavior
///
/// Deducts `amount` from `source` account and credits `amount` to
/// `target` account atomically. Returns a `Receipt` with the
/// transaction ID, timestamp, and balances of both accounts
/// post-transfer. If `source` balance is insufficient, no debit
/// occurs and `InsufficientFundsError` is returned. Transfer is
/// idempotent for the same `idempotency_key` within 24h — a
/// repeated request returns the original `Receipt`.
///
/// # Arguments
///
/// * `source` — Account ID to debit. Must be a valid, active account.
///   Returns `AccountNotFoundError` if account does not exist or is closed.
/// * `target` — Account ID to credit. Must be a valid, active account.
///   Returns `AccountNotFoundError` if account does not exist or is closed.
/// * `amount` — Non-negative `Decimal` to transfer. Must have at most
///   2 decimal places. Returns `ValueError` if negative or exceeds precision.
///
/// # Returns
///
/// `Receipt` containing transaction_id (String), timestamp (UTC DateTime),
/// source_balance (Decimal), target_balance (Decimal).
///
/// # Errors
///
/// - `InsufficientFundsError` — If source balance < amount after pending
///   holds. The `available_balance` field contains the actual available amount.
/// - `AccountNotFoundError` — If source or target account does not exist
///   or is closed. The `account_id` field identifies which account.
/// - `ValueError` — If amount is negative or has >2 decimal places.
/// - `DatabaseUnavailableError` — If the transaction database is unreachable.
///   No partial debit/credit occurs.
///
/// # Side Effects
///
/// - Writes a transaction record to the `transactions` table (after both
///   debit and credit succeed).
/// - Publishes a `funds.transferred` event to the message queue (after
///   transaction record is committed).
/// - Increments `transfer_count` metric on the source account.
/// - Logs at INFO on success, WARNING on insufficient funds.
///
fn transfer_funds(source: &str, target: &str, amount: Decimal) -> Result<Receipt, TransferError>
```

### Java — Javadoc

```java
/**
 * Transfer funds between accounts with transactional guarantees.
 *
 * <h3>Expected Behavior</h3>
 * <p>Deducts {@code amount} from {@code source} account and credits
 * {@code amount} to {@code target} account atomically. Returns a
 * Receipt with the transaction ID, timestamp, and balances of both
 * accounts post-transfer. If source balance is insufficient, no debit
 * occurs and InsufficientFundsException is thrown. Transfer is
 * idempotent for the same idempotencyKey within 24h — a repeated
 * request returns the original Receipt.</p>
 *
 * @param source Account ID to debit. Must be a valid, active account.
 *     Throws AccountNotFoundException if account does not exist or is closed.
 * @param target Account ID to credit. Must be a valid, active account.
 *     Throws AccountNotFoundException if account does not exist or is closed.
 * @param amount Non-negative BigDecimal to transfer. Must have at most
 *     2 decimal places. Throws IllegalArgumentException if negative or
 *     exceeds precision.
 * @return Receipt containing transactionId, timestamp (UTC),
 *     sourceBalance, and targetBalance.
 *
 * @throws InsufficientFundsException If source balance < amount after
 *     pending holds. The availableBalance field on the exception
 *     contains the actual available amount.
 * @throws AccountNotFoundException If source or target account does not
 *     exist or is closed. The accountId field identifies which account.
 * @throws IllegalArgumentException If amount is negative or has
 *     >2 decimal places.
 * @throws DatabaseUnavailableException If the transaction database is
 *     unreachable. No partial debit/credit occurs.
 *
 * @sideEffect Writes a transaction record to the transactions table
 *     (after both debit and credit succeed).
 * @sideEffect Publishes a funds.transferred event to the message queue
 *     (after transaction record is committed).
 * @sideEffect Increments transferCount metric on the source account.
 * @sideEffect Logs at INFO on success, WARNING on insufficient funds.
 */
public Receipt transferFunds(String source, String target, BigDecimal amount)
```

### C/C++ — Doxygen

```cpp
/**
 * @brief Transfer funds between accounts with transactional guarantees.
 *
 * @expectedBehavior Deducts @p amount from @p source account and credits
 * @p amount to @p target account atomically. Returns a Receipt with the
 * transaction ID, timestamp, and balances of both accounts post-transfer.
 * If source balance is insufficient, no debit occurs and
 * INSUFFICIENT_FUNDS is returned. Transfer is idempotent for the same
 * idempotency_key within 24h — a repeated request returns the original
 * Receipt.
 *
 * @param source Account ID to debit. Must be a valid, active account.
 *     Returns ACCOUNT_NOT_FOUND if account does not exist or is closed.
 * @param target Account ID to credit. Must be a valid, active account.
 *     Returns ACCOUNT_NOT_FOUND if account does not exist or is closed.
 * @param amount Non-negative Decimal to transfer. Must have at most
 *     2 decimal places. Returns VALUE_ERROR if negative or exceeds precision.
 * @param[out] receipt Pointer to Receipt struct to populate on success.
 *     Contains transaction_id, timestamp (UTC), source_balance,
 *     and target_balance.
 *
 * @return 0 on success, INSUFFICIENT_FUNDS if source balance < amount
 *     after pending holds, ACCOUNT_NOT_FOUND if source or target is
 *     invalid/closed, VALUE_ERROR if amount is invalid,
 *     DB_UNAVAILABLE if the transaction database is unreachable.
 *     No partial debit/credit occurs on any error.
 *
 * @throws None. All errors are returned via the return code.
 *
 * @sideeffect Writes a transaction record to the transactions table
 *     (after both debit and credit succeed).
 * @sideeffect Publishes a funds.transferred event to the message queue
 *     (after transaction record is committed).
 * @sideeffect Increments transfer_count metric on the source account.
 * @sideeffect Logs at INFO on success, WARNING on insufficient funds.
 */
int transfer_funds(const char* source, const char* target, Decimal amount, Receipt* receipt)
```

---

## Custom Tag Convention for Side Effects

Languages that lack a native "side effects" tag should use the following project-standard convention:

| Language | Side Effects Tag |
|----------|-----------------|
| Python | `Side Effects:` section in docstring |
| TypeScript/JavaScript | `@sideeffect` TSDoc tag |
| Go | `# Side Effects` section |
| Rust | `# Side Effects` section |
| Java | `@sideEffect` Javadoc tag |
| C/C++ | `@sideeffect` Doxygen tag |

---

## Atomic Commits Enforcement

Every commit must represent exactly one logical change. Mixed commits spanning unrelated features, fixes, or subsystems are forbidden.

### 1. One Logical Change Per Commit

A commit must represent **exactly one** of: a feature, a fix, a refactor, a docs update, or a chore.

If `git diff --stat` shows changes across unrelated subsystems (e.g. `services/ai-rust/` + `web/src/contexts/theme-context.tsx` + `docker-compose.yml`), it **must be split** into separate commits before pushing.

### 2. File Grouping Rules

Before committing, categorize modified files into logical groups and commit each group independently:

| Group | Examples |
|-------|----------|
| **Backend** | One microservice at a time (`ai-rust`, `core-rust`, `gateway-rust`, `ai-scraper`) |
| **Frontend** | One feature area at a time (theme, auth, SSE, admin pages, dashboard layout) |
| **Infra** | `docker-compose.yml`, DB migrations, `.env.example`, `.gitignore` |
| **CLI/Tools** | Standalone tooling or script changes |

Never mix backend + frontend + infra in the same commit unless the change is a single cross-cutting concern (e.g. a new env var that both backend and docker-compose need simultaneously).

### 3. How to Split Mixed Changes

1. Stage only files for the current logical group: `git add <file>...`
2. If a single file contains mixed changes for different concerns, use `git add -p` (patch mode) to stage hunks selectively.
3. If patch mode is too difficult or risky, edit the file to separate concerns, then stage and commit each group.
4. When reorganizing an existing mixed working tree, create a backup branch first: `git branch backup-<description>`

### 4. Untracked Files and Generated Artifacts

The following must **never** be committed:

- Screenshots (`screenshot_*.png`, `*.jpg` dumps)
- Temporary directories (`temp-*`, `.pytest_cache/`, `playwright-report/`)
- Build artifacts (`target/`, `node_modules/`, `.next/`, `out/`)
- Cache files (`*.tsbuildinfo`, `Cargo.lock` — already in `.gitignore`)
- Log files (`*.log`)

Before committing, verify no garbage is staged:
```bash
git status          # check untracked files
git diff --stat     # verify only intended files are modified
```

If untracked trash exists, add it to `.gitignore` or delete it. Reset accidentally modified artifacts with `git checkout -- <file>`.

### 5. Commit Message Format

Use **Conventional Commits** with imperative mood:

```
type(scope): description
```

- **Types**: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`, `perf`
- **Scope**: subsystem name (`ai`, `core`, `gateway`, `web`, `cli`, `infra`)
- **Description**: lowercase, imperative, no trailing period
- **Body** (optional): explain *why* for complex changes, keep to 72 chars/line

Examples:
```
feat(ai): add brand normalization endpoint
fix(web): read auth tokens from localStorage on every request
chore(infra): add ai-scraper-svc to docker-compose
```

### 6. Pre-Commit Checklist (Mandatory)

Before every `git commit`, confirm:

1. [ ] This commit touches **only one** logical concern.
2. [ ] No untracked/generated files are staged.
3. [ ] Commit message follows `type(scope): description` format.
4. [ ] `git diff --cached --stat` shows only files related to this single concern.
5. [ ] A `git log --oneline -5` preview makes sense as a readable history.

If any checkbox fails, stop and reorganize the commit before proceeding.
