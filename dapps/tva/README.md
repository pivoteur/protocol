# `tva`

Bidirectional BTC⇄UNDEAD auto-trader. Every cycle (every hour) checks all open pivots
against a live reverse quote and closes any that show a gain, then commits
remaining capital to new fixed-size pivots in both directions where
available.

## Usage

`$ tva [--dry-run]`

where:

* `[--dry-run]` checks only — never touches the keystore or sends a tx, no
  log entries written for opens or closes

> n.b: `TVA_WALLET_ADDRESS` and `TVA_KEYSTORE_PATH` must be set as
> environmental variables — distinct names from `arbitrage`'s
> `WALLET_ADDRESS`/`KEYSTORE_PATH`. Both binaries share a keystore-loading
> core, but each has its own wallet, so a shared env var name would let
> whichever was set last in a shell session silently win for both.
> `KEYSTORE_PASSWORD` is optional — set it for unattended runs (e.g. CI);
> omit it locally to be prompted interactively instead. `TVA_EXPECTED_WALLET`
> is optional — set it to hard-fail on a `TVA_WALLET_ADDRESS` mismatch
> instead of silently reading the wrong wallet's balance.

* [source](../../quizzes/src/quiz12/tva/mod.rs)

## State

All state is derived by replaying `tva-trades.log` from scratch on every
run — there's no separate persisted state file. Log lines start with a raw
Unix timestamp; to read one:

`$ date -d @1785876760`

converts it to your system's local time.

## Revisions

* 0.4.0, 2026-08-04: `TVA_WALLET_ADDRESS`/`TVA_KEYSTORE_PATH` — own env var
names, no longer shared with `arbitrage`; sequential pivot id fix (advances
per attempt, not only per success); explicit "No funds moved" on every
pre-transaction failure path
* 0.1.0: initial version — bidirectional BTC⇄UNDEAD pivot trading,
append-only trade log, gas tracking, dry-run mode
