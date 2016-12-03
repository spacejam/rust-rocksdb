// Copyright 2014 Tyler Neely
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

//! Rust wrapper for RocksDB.
//!
//! # Examples
//!
//! ```
//! use rocksdb::{Db, ReadOptions, WriteOptions};
//!
//! // Note: `db` is automatically closed at end of lifetime.
//! let db = Db::open_default("path/for/rocksdb/storage").unwrap();
//! db.put(b"my key", b"my value", &WriteOptions::default());
//! match db.get(b"my key", &ReadOptions::default()) {
//!     Ok(Some(value)) => println!("retrieved value {}", value.to_utf8().unwrap()),
//!     Ok(None) => println!("value not found"),
//!     Err(e) => println!("operational problem encountered: {}", e),
//! }
//! db.delete(b"my key", &WriteOptions::default()).unwrap();
//! ```
//!

extern crate libc;
extern crate librocksdb_sys as ffi;

#[macro_use]
mod ffi_util;

pub mod backup;
mod comparator;
mod db;
mod db_options;
pub mod merge_operator;
mod slice_transform;

pub use db::{DbCompactionStyle, DbCompressionType, DbIterator, DbRecoveryMode, DbVector,
             Direction, IteratorMode, WriteBatch, new_bloom_filter};

pub use merge_operator::MergeOperands;
use std::collections::BTreeMap;
use std::error;
use std::fmt;
use std::path::PathBuf;

/// A RocksDB database.
pub struct Db {
    inner: *mut ffi::rocksdb_t,
    cfs: BTreeMap<String, *mut ffi::rocksdb_column_family_handle_t>,
    path: PathBuf,
    #[allow(dead_code)]
    comparator: Option<Comparator>,
    #[allow(dead_code)]
    prefix_extractor: Option<SliceTransform>,
}

/// A RocksDB error.
#[derive(Debug, PartialEq)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Error {
        Error { message: message }
    }
}

impl From<Error> for String {
    fn from(e: Error) -> String {
        e.message
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.message.fmt(formatter)
    }
}

/// For configuring block-based file storage.
pub struct BlockBasedOptions {
    inner: *mut ffi::rocksdb_block_based_table_options_t,
}

/// Database-wide options around performance and behavior.
///
/// Please read [the official tuning guide](https://github.com/facebook/rocksdb/wiki/RocksDB-Tuning-Guide), and most importantly, measure performance under realistic workloads with realistic hardware.
///
/// # Examples
///
/// ```
/// use rocksdb::{Db, DbOptions};
/// use rocksdb::DbCompactionStyle;
///
/// fn badly_tuned_for_somebody_elses_disk() -> Db {
///    let path = "path/for/rocksdb/storage5";
///    let mut opts =  DbOptions::default();
///    opts.create_if_missing(true);
///    opts.set_max_open_files(10000);
///    opts.set_use_fsync(false);
///    opts.set_bytes_per_sync(8388608);
///    opts.set_disable_data_sync(false);
///    opts.optimize_for_point_lookup(1024);
///    opts.set_table_cache_num_shard_bits(6);
///    opts.set_max_write_buffer_number(32);
///    opts.set_write_buffer_size(536870912);
///    opts.set_target_file_size_base(1073741824);
///    opts.set_min_write_buffer_number_to_merge(4);
///    opts.set_level_zero_stop_writes_trigger(2000);
///    opts.set_level_zero_slowdown_writes_trigger(0);
///    opts.set_compaction_style(DbCompactionStyle::Universal);
///    opts.set_max_background_compactions(4);
///    opts.set_max_background_flushes(4);
///    opts.set_disable_auto_compactions(true);
///
///    Db::open(path, opts).unwrap()
/// }
/// ```
pub struct DbOptions {
    inner: *mut ffi::rocksdb_options_t,
    comparator: Option<Comparator>,
    prefix_extractor: Option<SliceTransform>,
}

/// Options for read operations.
pub struct ReadOptions {
    inner: *mut ffi::rocksdb_readoptions_t,
}

/// Options for write operations.
///
/// # Examples
///
/// Make an unsafe write of a batch:
///
/// ```
/// use rocksdb::{Db, WriteBatch, WriteOptions};
///
/// let db = Db::open_default("path/for/rocksdb/storage6").unwrap();
///
/// let mut batch = WriteBatch::default();
/// batch.put(b"my key", b"my value");
/// batch.put(b"key2", b"value2");
/// batch.put(b"key3", b"value3");
///
/// let mut write_options = WriteOptions::default();
/// write_options.set_sync(false);
/// write_options.disable_wal(true);
///
/// db.write(batch, &write_options);
/// ```
pub struct WriteOptions {
    inner: *mut ffi::rocksdb_writeoptions_t,
}

/// A key comparator.
pub struct Comparator {
    inner: *mut ffi::rocksdb_comparator_t,
}

/// A slice transform.
pub struct SliceTransform {
    inner: *mut ffi::rocksdb_slicetransform_t,
}

/// A consistent view of the database at the point of creation.
///
/// ```
/// use rocksdb::{Db, IteratorMode, ReadOptions};
///
/// let db = Db::open_default("path/for/rocksdb/storage3").unwrap();
/// let snapshot = db.snapshot(); // Creates a longer-term snapshot of the DB, but closed when goes out of scope
/// let mut iter = snapshot.iterator(IteratorMode::Start, &mut ReadOptions::default()); // Make as many iterators as you'd like from one snapshot
/// ```
///
pub struct Snapshot<'a> {
    db: &'a Db,
    inner: *const ffi::rocksdb_snapshot_t,
}
