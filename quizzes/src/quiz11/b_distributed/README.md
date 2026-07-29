# b_distributed 

A program that will automatically send a Telegram message, via `Robbie`, 
to each of Pivot Tech's investors, at a mass

`$ distributed <tsv_path> <send>`

where:

* `<tsv_path>` is the pathing to the <investors-test.tsv> file within: protocol/data/
* `<send>` is the "yes or no" option of if you want to use the Telgram bot, Robbie

* [src](quizzes/src/quiz11/b_distributed)

* `vesion 2.00`: workflow restructure, passing in a TSV file for `distributed` to read and 
gather all bits of data before constructing the costume messge for each investor. 

<investors-test.tsv> sample:
name	reinvested %	precentage	amount reinvested	amount distributed	primary	pivot	USD-value	number of pivots closed	tweet url	tx url	send?	flipped
α	100%	3.46%	14492	0	BTC	UNDEAD	$12.04	15	https://x.com/pivocateur/status/2069591552733712719		true	true
τ	0%	0.48%	0	2004	BTC	UNDEAD	$1.67	15	https://x.com/pivocateur/status/2069591552733712719	TX_URL_HERE	false	true
δ	100%	0.44%	1851	0	BTC	UNDEAD	$1.54	15	https://x.com/pivocateur/status/2069591552733712719		true	true
ι	100%	1.32%	5543	0	BTC	UNDEAD	$4.61	15	https://x.com/pivocateur/status/2069591552733712719		true	true
φ	100%	41.49%	173748	0	BTC	UNDEAD	$144.38	15	https://x.com/pivocateur/status/2069591552733712719		true	true
ψ	100%	41.50%	173765	0	BTC	UNDEAD	$144.40	15	https://x.com/pivocateur/status/2069591552733712719		true	true
π	100%	0.67%	2821	0	BTC	UNDEAD	$2.34	15	https://x.com/pivocateur/status/2069591552733712719		true	true
σ	0%	0.00%	0	0	BTC	UNDEAD	$0.00	15	https://x.com/pivocateur/status/2069591552733712719		true	true
ω	100%	0.39%	1614	0	BTC	UNDEAD	$1.34	15	https://x.com/pivocateur/status/2069591552733712719		true	true
γ	0%	10.25%	0	42910	BTC	UNDEAD	$35.66	15	https://x.com/pivocateur/status/2069591552733712719	TX_URL_HERE	false	true

## Revisions

* 2.00, 2026-07-29: Workflow change, this program reads from a TSV now instead of indivisual args
* 1.02, 2026-07-29: library-upgrades
* 1.01, 2026-07-07: use clap to parse arguments and for usage-documentation
* 1.00, 2026-05-19: release, and adding into the quizzes testing framework
