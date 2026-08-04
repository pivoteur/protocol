# `arbitrage`

Checks live wallet balance and a live KyberSwap quote for the BTC+ETH pivot
pool; if both clear, trades the full amount, either in one manual call or
in a batch pass over `calls.csv`. Never partial — 100% or nothing.

## Usage

`$ arbitrage trade <amount> <min_floor> [--flip] [--slippage-bps <bps>] [--dry-run]`

where:

* `<amount>`         is how much of the source token to trade, e.g. `0.1`
* `<min_floor>`      is the minimum acceptable amount back, e.g. `0.0028`
* `[--flip]`         reverses direction (default `BTC -> ETH`, flipped `ETH -> BTC`)
* `[--slippage-bps]` is the slippage tolerance in basis points, e.g. `50` (default)
* `[--dry-run]`      checks only — never touches the keystore or sends a tx

`$ arbitrage calls [--root-url <url>] [--slippage-bps <bps>] [--dry-run]`

where:

* `[--root-url]`     is where `calls.csv` is fetched from — falls back to
  the `PIVOT_URL` env var if omitted, e.g. `https://raw.githubusercontent.com/pivoteur/pivoteur.github.io`
* `[--slippage-bps]` same as above
* `[--dry-run]`      checks every row only — never touches the keystore or sends a tx

> n.b: `WALLET_ADDRESS` and `KEYSTORE_PATH` must be set as environmental
> variables — arbitrage's own names, distinct from `tva`'s
> `TVA_WALLET_ADDRESS`/`TVA_KEYSTORE_PATH`. Both binaries share a
> keystore-loading core (`libs::auto_trading`), but each has its own
> wallet, so the env var names can't be shared without one silently
> overwriting the other in the same shell session. `KEYSTORE_PASSWORD` is
> optional — set it for unattended runs (e.g. CI); omit it locally to be
> prompted interactively instead.

* [source](../../quizzes/src/quiz12/arbitrage/mod.rs)

## Revisions

* 0.10.1, 2026-08-04: `wallet_address_from_env`/`load_signer` now take the
env var name as a parameter rather than a hardcoded one — no behavior
change here (arbitrage still reads `WALLET_ADDRESS`/`KEYSTORE_PATH`), but
this is what makes distinct `tva`-specific names possible without
duplicating the shared signing core; every pre-transaction failure in
`load_signer` now says "No funds moved" explicitly
* 0.10.0: moved to `quizzes/src/quiz12/arbitrage/` (from `quiz11`); shared
live-fund-moving core (keystore load/verify, EIP-1559 fee buffering, exact-
amount approval, KyberSwap quoting, swap execution) extracted into
`libs/src/auto_trading.rs`, now used by both `arbitrage` and the new `tva`
binary — `mod.rs` reduced from ~786 to ~356 lines by delegating to it; gas
tracking added (`approve_exact_amount`/`send_swap_tx`/`execute_trade` now
return actual AVAX cost from real transaction receipts)
* 0.9.0, 2026-07-22: added `calls` subcommand — reads `calls.csv`, executes
any row the wallet can fully cover, refuses the rest; split into `trade`/ `calls` subcommands
* 0.8.0: simplified `--direction <normal|flipped>` down to a plain `--flip` flag
* 0.7.0: renamed `Direction` variants to `Normal`/`Flipped` to match the
`reinvested`/`distributed` `flipped` convention
* 0.5.0: bidirectional trading (`BTC -> ETH` and `ETH -> BTC`)
* 0.4.0: production-readiness pass — re-quote after keystore unlock,
EIP-1559 fee/gas buffering, dry-run mode, `f64` precision, HTTP timeouts, persistent trade log
* 0.3.0: initial working version — single direction, interactive keystore password only
