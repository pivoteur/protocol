# `a_reinvested` 

A program that will automatically send a Telegram message, via `Robbie`, 
to each of Pivot Tech's investors, at a mass

`$ reinvested <tsv_path> <send>`

where:

* `<tsv_path>` is the pathing to the <investors-test.tsv> file within: protocol/data/
* `<send>` is the "yes or no" option of if you want to use the Telgram bot, Robbie

* [src](mod.rs)

* `vesion 2.05`: workflow restructure, passing in a TSV file for `reinvested` to read and 
gather all bits of data before constructing the costume messge for each investor. 

[<investors-test.tsv> sample](../../../data/investors-test.tsv)

## Revisions

* 2.05, 2026-07-31: extracted shared Telegram/CSV logic (chat_id_for, parse_bool_cell, is_ragged_row, send_telegram, SendSpy) into libs::investor_rows; deduplicated shared test fixtures; cargo build and cargo test passing
* 2.00, 2026-07-29: Workflow change, this program reads from a TSV now instead of indivisual args 
* 1.02, 2026-07-29: library-upgrades
* 1.01, 2026-07-08: use clap to process arguments and for usage-documentation
* 1.00, 2026-05-18: release, and adding into the quizzes testing framework
