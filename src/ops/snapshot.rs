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

use crate::Snapshot;

/// This is an internal trait used to create and free a snapshot
pub trait SnapshotInternal {
    type DB: SnapshotInternal<DB = Self::DB>;

    unsafe fn create_snapshot(&self) -> Snapshot<Self::DB>;
    unsafe fn release_snapshot(&self, snapshot: &mut Snapshot<Self::DB>);
}

pub trait Snapshotable {
    type DB: SnapshotInternal<DB = Self::DB>;
    fn snapshot(&self) -> Snapshot<Self::DB>;
}

impl<T> Snapshotable for T
where
    T: SnapshotInternal<DB = T>,
{
    type DB = T;

    fn snapshot(&self) -> Snapshot<T> {
        unsafe { self.create_snapshot() }
    }
}
