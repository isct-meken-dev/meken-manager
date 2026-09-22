use crate::core::ParticipantId;
use crate::core::asset::Ticket;
use crate::core::workflow::{Workflow, WorkflowType};
use crate::prelude::*;
use chrono::NaiveDate;
use slotmap::{SlotMap, new_key_type};
use std::collections::BTreeSet;

errors! {
    pub enum TaskError {} {
        NotFound => "task not found",
        NoAuthority => "",
    }
    pub type TaskResult<T>;
}

#[bulkderive(Persist!)]
#[derive(Default)]
pub struct TaskList {
    pool: SlotMap<TaskKey, Task>,
    keys: Vec<TaskKey>,
}
impl TaskList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn task(&self, key: TaskKey) -> TaskResult<&Task> {
        self.pool.get(key).ok_or(TaskError::NotFound)
    }
    pub fn task_mut(&mut self, key: TaskKey) -> TaskResult<&mut Task> {
        self.pool.get_mut(key).ok_or(TaskError::NotFound)
    }
    pub fn add_task(&mut self, ty: WorkflowType) {
        let key = self.pool.insert(Task::new(ty));
        self.keys.push(key);
    }

    pub fn keys_iter(&self) -> impl Iterator<Item = TaskKey> {
        self.keys.iter().map(|k| *k)
    }
    pub fn key(&self, index: usize) -> TaskResult<TaskKey> {
        self.keys.get(index).ok_or(TaskError::NotFound).map(|k| *k)
    }
    pub fn remove_task(&mut self, key: TaskKey) -> TaskResult<Task> {
        self.keys
            .remove(self.keys.iter().position(|&k| k == key).unwrap());
        self.pool.remove(key).ok_or(TaskError::NotFound)
    }
}

#[bulkderive(Persist!)]
pub struct Task {
    workflow: Workflow,
    deadline: Option<NaiveDate>,
    participant: Option<ParticipantId>,
    tickets: Vec<Ticket>,
}
impl Task {
    pub fn new(ty: WorkflowType) -> Self {
        Self {
            workflow: Workflow::new(ty),
            deadline: None,
            participant: None,
            tickets: Vec::new(),
        }
    }

    pub fn workflow(&self) -> &Workflow {
        &self.workflow
    }
    pub fn workflow_mut(&mut self) -> &mut Workflow {
        &mut self.workflow
    }
    pub fn workflow_type(&self) -> WorkflowType {
        self.workflow.get_type()
    }

    pub fn participant(&self) -> Option<ParticipantId> {
        self.participant
    }
    pub fn reset_participant(&mut self) {
        self.participant.take();
    }
    pub fn set_participant(&mut self, participant: ParticipantId) {
        self.participant.replace(participant);
    }

    pub fn deadline(&self) -> Option<NaiveDate> {
        self.deadline
    }
    pub fn reset_deadline(&mut self) {
        self.deadline = None;
    }
    pub fn set_deadline(&mut self, date: NaiveDate) {
        self.deadline = Some(date);
    }

    pub(in crate::core) fn tickets(&self) -> &Vec<Ticket> {
        &self.tickets
    }
    pub(in crate::core) fn tickets_mut(&mut self) -> &mut Vec<Ticket> {
        &mut self.tickets
    }

    pub fn split(&mut self, indices: BTreeSet<usize>) -> TaskResult<Self> {
        if indices.range(self.tickets.len()..).next().is_some() {
            Err(TaskError::NotFound)
        } else {
            let tickets = indices
                .into_iter()
                .rev()
                .map(|i| self.tickets.remove(i))
                .rev()
                .collect();
            Ok(Self {
                tickets,
                workflow: self.workflow.clone(),
                ..*self
            })
        }
    }

    pub fn can_marge(&self, other: &Self) -> bool {
        self.deadline != other.deadline
            || self.participant != other.participant
            || self.workflow != other.workflow
    }
    pub fn merge(&mut self, other: Self) -> Result<(), Self> {
        if self.can_marge(&other) {
            Err(other)
        } else {
            self.tickets.extend(other.tickets);
            Ok(())
        }
    }

    pub fn can_discard(&self) -> bool {
        self.participant.is_some() || self.deadline.is_some()
    }
    pub(in crate::core) fn discard_unchecked(self) -> Vec<Ticket> {
        self.tickets
    }
}

new_key_type! {
    pub struct TaskKey;
}
