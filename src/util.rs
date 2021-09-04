pub type DynFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output=T>>>;

pub fn oneshot<T: 'static>() -> (async_oneshot::Sender<T>, DynFuture<T>) {
    let (sender, receiver) = async_oneshot::oneshot();
    let receiver = Box::pin(
        async move {
            match receiver.await {
                Ok(result) => result,
                Err(_) => {
                    // The receiver deconnected. Wait infinitely. The receiver does not care if it disconnected,
                    // it only wants to know if something was sended (like registering a callback)
                    let () = std::future::pending().await;
                    unreachable!();
                }
            }
        }
    );

    (sender, receiver)
}