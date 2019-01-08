# tokio-balance

Note: Not associated with the [tokio](http://tokio.rs/) project.

Exposes a `Balancer` struct that implements `Sink<T,E>`. It is constructed with a Vec
of `Sink<T,E>` and round-robin balances any sunk Item to the first Sink that accepts it
(i.e. is not full). This can be useful to e.g. balance items across multiple tokio channels
or db connections etc.
