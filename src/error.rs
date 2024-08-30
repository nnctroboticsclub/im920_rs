use srobo_base::utils::fifo;

#[derive(Debug)]
pub enum Error<E> {
    SerialError(E),
    Fifo(fifo::Error),
    Timeout,
    OperationFailed,
}
