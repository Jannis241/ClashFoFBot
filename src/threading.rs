use crate::prelude::*;

/// Automatischer Feldzugriff – wird intern verwendet
pub trait AutoThread: Send + 'static {
    fn run(&mut self); // Benutzerdefiniert
    fn set_field_any(&mut self, field: &str, value: Box<dyn std::any::Any>) -> bool;
    fn get_field_any(&self, field: &str) -> Option<&dyn std::any::Any>;
}

#[macro_export]
macro_rules! auto_set_field {
    ($field_name:literal, |$arg:ident : $ty:ty| $body:expr) => {{
        if field == $field_name {
            if let Ok($arg) = value.downcast::<$ty>() {
                $body;
                return true;
            }
        }
    }};
}

/// Thread-Wrapper
pub struct WorkerHandle<T: AutoThread> {
    state: Arc<Mutex<T>>,
    running: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}

impl<T: AutoThread> WorkerHandle<T> {
    pub fn new(mut obj: T) -> Self {
        let state = Arc::new(Mutex::new(obj));
        let state_thread = Arc::clone(&state);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let handle = thread::spawn(move || {
            let mut obj = state_thread.lock().unwrap();
            obj.run();
            running_clone.store(false, Ordering::SeqCst);
        });

        Self {
            state,
            running,
            join_handle: Some(handle),
        }
    }

    pub fn set_field<U: 'static>(&self, key: &str, value: U) {
        if let Ok(mut state) = self.state.lock() {
            let _ = state.set_field_any(key, Box::new(value));
        }
    }

    pub fn get_output<U: 'static>(&self, key: &str) -> Option<U>
    where
        U: Clone,
    {
        let data = self.state.lock().ok()?;
        let any = data.get_field_any(key)?;
        any.downcast_ref::<U>().cloned()
    }

    pub fn update<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let state = self.state.lock().unwrap();
        f(&*state)
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn stop(mut self) {
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

#[macro_export]
macro_rules! impl_auto_fields {
    ($struct_name:ident, { $($field:ident),* $(,)? }) => {
        impl AutoThread for $struct_name {
            fn run(&mut self) {
                // Standardmäßig nichts tun
            }

            fn set_field(&mut self, key: &str, value: &str) {
                match key {
                    $(
                        stringify!($field) => {
                            if let Ok(parsed) = value.parse() {
                                self.$field = parsed;
                            }
                        }
                    )*
                    _ => {}
                }
            }

            fn get_output(&self, key: &str) -> Option<String> {
                match key {
                    $(
                        stringify!($field) => Some(self.$field.to_string()),
                    )*
                    _ => None,
                }
            }
        }
    };
}
