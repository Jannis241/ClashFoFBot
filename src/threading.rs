use std::{
    any::Any,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub trait AutoThread: Send + 'static {
    fn run(&mut self);
    fn handle_field_set(&mut self, field: &str, value: Box<dyn Any + Send>);
    fn handle_field_get(&self, field: &str) -> Option<Box<dyn Any + Send>>;
}

pub struct WorkerHandle<T: AutoThread> {
    input_tx: Sender<(
        String,
        Box<dyn Any + Send>,
        Option<Sender<Box<dyn Any + Send>>>,
    )>,
    stop_tx: Sender<()>,
    join_handle: JoinHandle<()>,
    _marker: std::marker::PhantomData<T>,
    running: Arc<AtomicBool>,
}

impl<T: AutoThread> WorkerHandle<T> {
    pub fn start(mut inner: T, should_loop: bool) -> Self {
        let (input_tx, input_rx): (
            Sender<(
                String,
                Box<dyn Any + Send>,
                Option<Sender<Box<dyn Any + Send>>>,
            )>,
            Receiver<_>,
        ) = channel();
        let (stop_tx, stop_rx) = channel();

        let join_handle = thread::spawn(move || {
            if should_loop {
                loop {
                    if let Ok(()) = stop_rx.try_recv() {
                        break;
                    }

                    while let Ok((key, value, resp)) = input_rx.try_recv() {
                        if let Some(resp) = resp {
                            let result = inner.handle_field_get(&key);
                            let _ = resp.send(result.unwrap_or_else(|| Box::new(())));
                        } else {
                            inner.handle_field_set(&key, value);
                        }
                    }

                    inner.run();
                    thread::sleep(Duration::from_millis(1)); // winzige Pause
                }
            } else {
                while let Ok((key, value, resp)) = input_rx.try_recv() {
                    if let Some(resp) = resp {
                        let result = inner.handle_field_get(&key);
                        let _ = resp.send(result.unwrap_or_else(|| Box::new(())));
                    } else {
                        inner.handle_field_set(&key, value);
                    }
                }

                inner.run();
                thread::sleep(Duration::from_millis(1)); // winzige Pause
            }
        });

        Self {
            input_tx,
            stop_tx,
            join_handle,
            running: Arc::new(AtomicBool::new(true)),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn stop(self) {
        let _ = self.stop_tx.send(());
        let _ = self.join_handle.join();
        let _ = self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn set_field<U: Any + Send>(&self, key: &str, value: U) {
        let _ = self.input_tx.send((key.to_string(), Box::new(value), None));
    }

    pub fn get_field<U: Any + Send>(&self, key: &str) -> Option<U> {
        let (tx, rx) = channel();
        let _ = self
            .input_tx
            .send((key.to_string(), Box::new(()), Some(tx)));
        rx.recv().ok()?.downcast::<U>().ok().map(|b| *b)
    }
}

#[macro_export]
macro_rules! auto_set_field {
    ($self_:ident, $key:expr, $value:expr, { $($field_name:literal => $field:ident : $ty:ty),* $(,)? }) => {
        match $key {
            $(
                $field_name => {
                    if let Ok(val) = $value.downcast::<$ty>() {
                        $self_.$field = *val;
                        return;
                    }
                }
            )*
            _ => {}
        }
    };
}

#[macro_export]
macro_rules! auto_get_field {
    ($self_:ident, $key:expr, { $($field_name:literal => $field:ident : $ty:ty),* $(,)? }) => {{
        match $key {
            $(
                $field_name => {
                    return Some(Box::new($self_.$field.clone()) as Box<dyn std::any::Any + Send>);
                }
            )*
            _ => return None,
        }
    }};
}
