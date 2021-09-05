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

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    fn setTimeout(closure: &wasm_bindgen::prelude::Closure<dyn FnMut()>, time: u32) -> u32;
    fn clearTimeout(handle: u32);
}

pub struct Timeout {
    handle: u32,
    _cb: wasm_bindgen::prelude::Closure<dyn FnMut()>
}

impl Timeout {
    pub fn sleep(time: u32, closure: wasm_bindgen::prelude::Closure<dyn FnMut()>) -> Self {
        let handle = setTimeout(&closure, time);
        Timeout {
            handle,
            _cb: closure
        }
    }
}

impl Drop for Timeout {
    fn drop(&mut self) {
        clearTimeout(self.handle);
    }
}

pub fn sleep(time: u32) -> DynFuture<()> {
    let (mut sender, receiver) = async_oneshot::oneshot();
    let closure = wasm_bindgen::prelude::Closure::once(move || {
        let _ = sender.send(());
    });

    let timeout = Timeout::sleep(time, closure);

    Box::pin(async move {
        let result = receiver.await;
        drop(timeout);
        match result {
            Ok(result) => {
                result
            },
            Err(_) => {
                let () = std::future::pending().await;
                unreachable!();
            }
        }
    })
}

pub struct EventChannel<T> {
    inner: std::rc::Rc<std::cell::RefCell<EventChannelInner<T>>>
}

struct EventChannelInner<T> {
    senders: Vec<async_oneshot::Sender<T>>
}

impl<T: Clone> EventChannelInner<T> {
    pub fn new() -> Self {
        Self {
            senders: Vec::new()
        }
    }

    pub fn send(&mut self, value: T) {
        for mut sender in self.senders.drain(..) {
            let _ = sender.send(value.clone());
        }
    }
}

impl<T: Clone + 'static> EventChannel<T> {
    pub fn new() -> Self {
        Self {
            inner: std::rc::Rc::new(std::cell::RefCell::new(EventChannelInner::new()))
        }
    }

    pub fn _send(&self, value: T) {
        self.inner.borrow_mut().send(value)
    }

    pub fn callback(&self) -> yew::Callback<T> {
        let inner = self.inner.clone();
        (move |value| {
            inner.borrow_mut().send(value);
        }).into()
    }

    pub fn receive(&self) -> DynFuture<T> {
        let (sender, receiver) = oneshot();
        self.inner.borrow_mut().senders.push(sender);
        receiver
    }
}

mod spawn {
    pin_project_lite::pin_project! {
        struct CancellableFuture<F> {
            #[pin]
            f: F,
            cancelled: std::rc::Rc<std::cell::Cell<bool>>
        }
    }

    impl<F: std::future::Future<Output=()>> std::future::Future for CancellableFuture<F> {
        type Output = ();

        fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<()> {
            let this = self.project();
            if this.cancelled.get() {
                return std::task::Poll::Ready(())
            }

            this.f.poll(cx)
        }
    }

    pub struct SpawnHandle(std::rc::Rc<std::cell::Cell<bool>>);

    impl SpawnHandle {
        pub fn new() -> Self {
            SpawnHandle(Default::default())
        }

        pub fn cancel(self) {
            drop(self);
        }
    }

    impl Drop for SpawnHandle {
        fn drop(&mut self) {
            self.0.set(true);
            // TODO: Wake the future if it wasn't cancelled before
        }
    }
    
    pub fn spawn_local_cancellable(f: impl std::future::Future<Output=()> + 'static) -> SpawnHandle {
        let handle = SpawnHandle::new();
        let f = CancellableFuture {
            f,
            cancelled: handle.0.clone()
        };

        wasm_bindgen_futures::spawn_local(f);
        handle
    }
}

pub use spawn::{SpawnHandle, spawn_local_cancellable};