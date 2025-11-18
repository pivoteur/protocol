# b_compute_close

Now that we have the [open pivots](../../quiz01) in hand and [today's 
quotes](../a_quotes), now let's simulate what a close pivot would result in,
then, give a go/no-go-signal on the close.

We'll take some steps to get there.

First, let's create a library-call to fetch the open pivots:

* library [fetchers](../../../libs/src/fetchers.rs#L16)

![Fetch open pivots](imgs/01-capture-pivots.png)

Next, let's fetch today's quotes

* library [fetchers](../../../libs/src/fetchers.rs#L68), again.

![... adding today's quotes](imgs/02-add-quotes.png)
![embed date with quotes](imgs/03-embed-date.png)

... We also embed the date-information with the quotes, as they are dependent
types.
