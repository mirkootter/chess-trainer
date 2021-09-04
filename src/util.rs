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