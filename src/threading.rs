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

use std::collections::HashMap;

// Trait AnyClone mit as_any für Downcast-Referenzen
pub trait AnyClone: Any + Send {
    fn clone_box(&self) -> Box<dyn AnyClone>;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> AnyClone for T
where
    T: Any + Send + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn AnyClone> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Funktion für Box-Downcast
pub fn downcast_box<T: Any>(b: Box<dyn AnyClone>) -> Result<Box<T>, Box<dyn AnyClone>> {
    if b.as_any().is::<T>() {
        // Wir wandeln den Box-Rohzeiger um
        let raw = Box::into_raw(b);
        let raw = raw as *mut T;
        unsafe { Ok(Box::from_raw(raw)) }
    } else {
        Err(b)
    }
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
    cache: Mutex<HashMap<String, Box<dyn AnyClone>>>,

    // 🧠 Feld: Ausstehende Feld-Abfragen
    pending_rx: Arc<Mutex<HashMap<String, Receiver<Box<dyn Any + Send>>>>>,
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
            cache: Mutex::new(HashMap::new()),
            running: Arc::new(AtomicBool::new(true)),
            pending_rx: Arc::new(Mutex::new(HashMap::new())),
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

    pub fn poll_field<U: Any + Send + Clone>(&self, key: &str) -> Option<U> {
        // 2. Pending check
        let mut pending = self.pending_rx.lock().unwrap();
        if let Some(rx) = pending.get(key) {
            match rx.try_recv() {
                Ok(value) => {
                    // versuche Downcast
                    if let Ok(boxed_val) = value.downcast::<U>() {
                        let cloned: U = (*boxed_val).clone(); // aus Box<U> -> U
                        self.cache.lock().unwrap().insert(
                            key.to_string(),
                            Box::new(cloned.clone()) as Box<dyn AnyClone>,
                        );
                        pending.remove(key);
                        return Some(cloned);
                    } else {
                        pending.remove(key);
                        return None;
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => return None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Channel ist tot, entferne pending
                    pending.remove(key);
                    return None;
                }
            }
        }

        // 3. Neue Anfrage starten
        let (tx, rx) = channel();
        let _ = self
            .input_tx
            .send((key.to_string(), Box::new(()), Some(tx)));
        pending.insert(key.to_string(), rx);

        // 1. Cache check
        if let Some(cached_value) = self.cache.lock().unwrap().get(key) {
            if let Some(value) = cached_value.as_ref().as_any().downcast_ref::<U>() {
                return Some(value.clone());
            }
        }

        None
    }

    pub fn get_field_async(&self, key: &str) -> Receiver<Box<dyn Any + Send>> {
        let (tx, rx) = channel();
        let _ = self
            .input_tx
            .send((key.to_string(), Box::new(()), Some(tx)));
        rx
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
