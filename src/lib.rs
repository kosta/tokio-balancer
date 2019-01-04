use core::ops::IndexMut;
use futures::sink::Sink;
use futures::{Async, AsyncSink};

type StartSend<T, E> = Result<AsyncSink<T>, E>;
type Poll<T, E> = Result<Async<T>, E>;

pub struct Balancer<List, Item, Error>
where
    List: IndexMut<usize, Output = Sink<SinkItem = Item, SinkError = Error>>,
{
    n: usize,
    i: usize,
    list: List,
}

impl<List, Item, Error> Balancer<List, Item, Error>
where
    List: IndexMut<usize, Output = Sink<SinkItem = Item, SinkError = Error>>,
{
    pub fn new(n: usize, list: List) -> Balancer<List, Item, Error> {
        Balancer { i: 0, n, list }
    }
}

// impl<T, Item, Error> Balancer<Vec<T>, Item, Error>
// where
//     T: Sink<SinkItem = Item, SinkError = Error>,
// {
//     pub fn new_from_vec(v: Vec<T>) -> Balancer<Vec<T>, Item, Error>
//     {
//         Balancer {
//             i: 0,
//             n: v.len(),
//             list: v,
//         }
//     }
// }

impl<List, Item, Error> Sink for Balancer<List, Item, Error>
where
    List: IndexMut<usize, Output = Sink<SinkItem = Item, SinkError = Error>>,
{
    type SinkItem = Item;
    type SinkError = Error;

    fn start_send(
        &mut self,
        mut item: Self::SinkItem,
    ) -> StartSend<Self::SinkItem, Self::SinkError> {
        for _ in 0..self.n {
            let sink = &mut self.list[self.i];
            self.i += 1;

            match sink.start_send(item) {
                Ok(AsyncSink::NotReady(rejected_item)) => {
                    item = rejected_item;
                }
                final_result => {
                    //Ok(Ready) or an error; in both cases we're done
                    return final_result;
                }
            }
        }
        Ok(AsyncSink::NotReady(item))
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        // TODO: does it make sense to use/increment i here?
        let mut all_ready = true;
        for _ in 0..self.n {
            let sink = &mut self.list[self.i];
            self.i += 1;

            all_ready = sink.poll_complete()?.is_ready() && all_ready;
        }
        if all_ready {
            Ok(Async::Ready(()))
        } else {
            Ok(Async::NotReady)
        }
    }
}

#[cfg(test)]
mod tests {

    use futures::stream::Stream;
    use futures::future::ok;
    use futures::sync::mpsc::channel;
    use super::Balancer;

    const BUF_SIZE: usize = 1024;
    const N: usize = 8;

    #[test]
    fn it_works() {
        let (sink_tx, sink_rx) = channel::<u32>(BUF_SIZE);
        let mut balanced = Vec::new();
        let mut folded = Vec::new();
        for _ in 0..N {
            let (tx, rx) = channel::<u32>(BUF_SIZE);
            balanced.push(tx);
            folded.push(rx.fold(0, |a, b| ok(a+b)))
        }
        let balancer = Balancer::new(balanced.len(), balanced);
    }
}
