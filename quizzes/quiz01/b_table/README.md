# b_table

Parses raw data from REST endpoint, reifying them to pivots.

* [Problem statement](https://x.com/pivocateur/status/1987968638150738406)
* [Questions on parsed data](https://x.com/pivocateur/status/1987986705459384621)

Step 1: Create Pivot-structures, reify data

* we start by 
<a href="https://github.com/pivoteur/protocol/blob/integrate-opened-dt/quizzes/quiz01/b_table/src/main.rs">parsing
the opened-date</a> into the Pivot-structures.

![Parsed dates in Pivot structures](imgs/01-parsed-dates.png)

The result shows pivots with parsed-dates.

* Next we 
<a href="https://github.com/pivoteur/protocol/blob/pivot-header/quizzes/quiz01/b_table/src/main.rs">parse
the entire header</a>.

![Parsed header](imgs/02-parsed-header.png)

By parsing the entire header, we now can partition pivots into active and closed
groups, allowing us to focus on open pivots, only.

* Now, let's parse everything. I
<a href="https://github.com/pivoteur/protocol/blob/pivot-asset/quizzes/quiz01/b_table/src/main.rs">parse
assets</a>, which completes the reification of pivots.

![Data-driven pivots](imgs/03-parse-pivot.png)

Real pivots from real data.

* Finally, a touch of clean-up: the machine-representation is hard for me
to digest, so I 
<a href="https://github.com/pivoteur/protocol/blob/pivot-as-csv/quizzes/quiz01/b_table/src/main.rs">output 
the results as CSV</a> and total pivots and their amounts.

![Pivots as CSV](imgs/04-csv-and-amounts.png)

Pivots as CSV.
