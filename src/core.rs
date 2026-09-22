pub mod asset;
pub mod cut;
pub mod task;
pub mod workflow;

use crate::core::asset::{AssetKey, AssetPool};
use crate::core::cut::CutSequence;
use crate::core::task::{TaskKey, TaskList};
use crate::prelude::*;
use bimap::BiBTreeMap;
use slotmap::{SlotMap, new_key_type};
use std::collections::BTreeMap;

errors! {
    pub enum ParticipantError {} {
        NotFound => "participant not found"
    }
    pub type ParticipantResult<T>;
}

errors! {
    pub enum ProjectError {
        Asset => asset::AssetError,
        Cut => cut::CutError,
        Task => task::TaskError,
        Workflow => workflow::WorkflowError,
        Participant => ParticipantError,
    } {
        IdDuplicated => "id duplicated",
        NotFound => "project not found",
    }
    pub type ProjectResult<T>;
}

#[bulkderive(Persist!)]
#[derive(Default)]
pub struct ProjectRegistry {
    projects: SlotMap<ProjectKey, Project>,
    server_map: BiBTreeMap<ServerId, ProjectKey>,
    drive_map: BiBTreeMap<DriveId, ProjectKey>,
}
impl ProjectRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_project(
        &mut self,
        server_id: ServerId,
        drive_id: DriveId,
    ) -> ProjectResult<ProjectKey> {
        if self.server_map.contains_left(&server_id) || self.drive_map.contains_left(&drive_id) {
            return Err(ProjectError::IdDuplicated);
        }
        let key = self.projects.insert(Project::new());
        self.server_map.insert(server_id, key);
        self.drive_map.insert(drive_id, key);
        Ok(key)
    }
    pub fn delete_project(&mut self, key: ProjectKey) -> ProjectResult<()> {
        self.server_map
            .remove_by_right(&key)
            .ok_or(ProjectError::NotFound)?;
        self.drive_map
            .remove_by_right(&key)
            .ok_or(ProjectError::NotFound)?;
        self.projects.remove(key).ok_or(ProjectError::NotFound)?;
        Ok(())
    }

    pub fn project(&self, key: ProjectKey) -> ProjectResult<&Project> {
        self.projects.get(key).ok_or(ProjectError::NotFound)
    }
    pub fn project_mut(&mut self, key: ProjectKey) -> ProjectResult<&mut Project> {
        self.projects.get_mut(key).ok_or(ProjectError::NotFound)
    }

    pub fn server_id(&self, key: ProjectKey) -> ProjectResult<ServerId> {
        self.server_map
            .get_by_right(&key)
            .ok_or(ProjectError::NotFound)
            .map(|id| id.clone())
    }
    pub fn key_from_server(&self, server_id: &ServerId) -> ProjectResult<ProjectKey> {
        self.server_map
            .get_by_left(server_id)
            .ok_or(ProjectError::NotFound)
            .map(|id| id.clone())
    }
    pub fn drive_id(&self, key: ProjectKey) -> ProjectResult<DriveId> {
        self.drive_map
            .get_by_right(&key)
            .ok_or(ProjectError::NotFound)
            .map(|id| id.clone())
    }
    pub fn key_from_drive(&self, drive_id: &DriveId) -> ProjectResult<ProjectKey> {
        self.drive_map
            .get_by_left(drive_id)
            .ok_or(ProjectError::NotFound)
            .map(|id| id.clone())
    }
}

new_key_type! {
    pub struct ProjectKey;
}

#[bulkderive(Persist!)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ServerId(u64);
impl ServerId {
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub fn inner(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, serde::Serialize, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct DriveId(&'static str);
impl DriveId {
    pub fn new(raw: String) -> Self {
        Self(internment::Intern::new(raw).as_ref())
    }
    pub fn inner(&self) -> &'static str {
        self.0
    }
}
impl<'de> serde::Deserialize<'de> for DriveId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(String::deserialize(deserializer)?))
    }
}

#[bulkderive(Persist!)]
#[derive(Default)]
pub struct Project {
    cuts: CutSequence,
    participants: BTreeMap<ParticipantId, Participant>,
    assets: AssetPool,
    tasks: TaskList,
}
impl Project {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tasks(&self) -> &TaskList {
        &self.tasks
    }
    pub fn tasks_mut(&mut self) -> &mut TaskList {
        &mut self.tasks
    }

    pub fn participant(&self, id: &ParticipantId) -> ProjectResult<&Participant> {
        Ok(self
            .participants
            .get(id)
            .ok_or(ParticipantError::NotFound)?)
    }
    pub fn set_participant(&mut self, id: ParticipantId, participant: Participant) {
        self.participants.insert(id, participant);
    }
    pub fn delete_participant(&mut self, id: ParticipantId) -> ProjectResult<()> {
        self.participants
            .remove(&id)
            .ok_or(ParticipantError::NotFound)?;
        Ok(())
    }

    pub fn stack_asset(&mut self, asset_key: AssetKey, task_key: TaskKey) -> ProjectResult<()> {
        if self.assets.asset(asset_key)?.workflow_type()
            == self.tasks.task(task_key)?.workflow_type()
        {
            let ticket = self.assets.take_ticket(asset_key)?;
            self.tasks.task_mut(task_key)?.tickets_mut().push(ticket);
            Ok(())
        } else {
            Err(workflow::WorkflowError::TypeMismatch)?
        }
    }
    pub fn discard_task(&mut self, key: TaskKey) -> ProjectResult<()> {
        if self.tasks.task(key)?.can_discard() {
            self.tasks
                .remove_task(key)?
                .discard_unchecked()
                .into_iter()
                .for_each(|ticket| self.assets.return_ticket(ticket));
            Ok(())
        } else {
            Err(task::TaskError::NoAuthority)?
        }
    }
}

#[bulkderive(Persist!)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ParticipantId(u64);
impl ParticipantId {
    pub fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub fn inner(&self) -> u64 {
        self.0
    }
}

#[bulkderive(Persist!)]
pub struct Participant {
    name: String,
}
