use jarust_core::prelude::*;
use jarust_rt::JaTask;
use std::ops::Deref;

pub struct LegacyVideoRoomHandle {
    handle: JaHandle,
    task: Option<JaTask>,
}

impl PluginTask for LegacyVideoRoomHandle {
    fn assign_task(&mut self, task: JaTask) {
        self.task = Some(task);
    }

    fn cancel_task(&mut self) {
        if let Some(task) = self.task.take() {
            task.cancel()
        };
    }
}

impl From<JaHandle> for LegacyVideoRoomHandle {
    fn from(handle: JaHandle) -> Self {
        Self { handle, task: None }
    }
}

impl Deref for LegacyVideoRoomHandle {
    type Target = JaHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Drop for LegacyVideoRoomHandle {
    fn drop(&mut self) {
        self.cancel_task();
    }
}
