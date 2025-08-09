// Crate-intern
pub use crate::*;
pub use FofError::*;

// Drittanbieter-Bibliotheken
pub use chrono::Local;
pub use csv::Reader;
pub use device_query::{DeviceState, Keycode};
pub use eframe;
pub use eframe::egui;
pub use eframe::egui::{Color32, RichText, StrokeKind};
pub use image::{io::Reader as ImageReader, ImageBuffer, Rgba, RgbaImage};
pub use rand::Rng;
pub use rfd;
pub use screenshots::Screen;
pub use serde::{Deserialize, Serialize};
use std::collections::HashSet;
pub use strum::IntoEnumIterator;
pub use strum_macros::{Display, EnumIter};

// Standardbibliothek – collections & errors
pub use std::collections::HashMap;
pub use std::error::Error;
pub use std::fmt::{Debug, Display};
pub use std::fs::{self, copy, read_to_string, remove_file, write, File, OpenOptions};
pub use std::io::{BufReader, BufWriter, Read, Write};
pub use std::os::unix::ffi::OsStrExt;
pub use std::path::{Path, PathBuf};
pub use std::process::Command;
pub use std::result::Result;
pub use std::str::FromStr;
pub use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
pub use std::thread::{self, JoinHandle};
pub use std::time::Duration;

pub use std::io;
use time::*;
