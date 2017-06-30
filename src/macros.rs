
macro_rules! iotry {
    ($e: expr) => ( $e.map_err(|err| Error::new(ErrorKind::Other, err))?)
}
