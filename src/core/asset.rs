use crate::core::workflow::WorkflowType;
use crate::prelude::*;
use slotmap::{SlotMap, new_key_type};

errors! {
    pub enum AssetError {} {
        NotFound => "asset not found",
        InUse => "asset is in use",
    }
    pub type AssetResult<T>;
}

#[bulkderive(Persist!)]
pub struct AssetPool(SlotMap<AssetKey, (Option<Ticket>, Asset)>);
impl AssetPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn asset(&self, key: AssetKey) -> AssetResult<&Asset> {
        Ok(&self.0.get(key).ok_or(AssetError::NotFound)?.1)
    }
    pub fn asset_mut(&mut self, key: AssetKey) -> AssetResult<&mut Asset> {
        Ok(&mut self.0.get_mut(key).ok_or(AssetError::NotFound)?.1)
    }

    pub fn add_asset(&mut self, workflow_type: WorkflowType, name: String) -> AssetKey {
        self.0.insert_with_key(|key| {
            (
                Some(Ticket(key)),
                Asset {
                    workflow_type,
                    name,
                },
            )
        })
    }
    pub fn delete_asset(&mut self, key: AssetKey) -> AssetResult<()> {
        match self.0.get(key) {
            None => Err(AssetError::NotFound),
            Some((None, _)) => Err(AssetError::InUse),
            Some((Some(_), _)) => {
                self.0.remove(key);
                Ok(())
            }
        }
    }
    pub(in crate::core) fn take_ticket(&mut self, key: AssetKey) -> AssetResult<Ticket> {
        match self.0.get_mut(key) {
            None => Err(AssetError::NotFound),
            Some((inner, _)) => match inner.take() {
                Some(ticket) => Ok(ticket),
                None => Err(AssetError::InUse),
            },
        }
    }
    pub(in crate::core) fn return_ticket(&mut self, ticket: Ticket) {
        if let Some(_) = self.0.get_mut(ticket.0).unwrap().0.replace(ticket) {
            panic!("ticket may be duplicated!")
        }
    }
}
impl Default for AssetPool {
    fn default() -> Self {
        Self(SlotMap::with_key())
    }
}

#[bulkderive(Persist!)]
pub struct Asset {
    workflow_type: WorkflowType,
    name: String,
}
impl Asset {
    pub fn workflow_type(&self) -> WorkflowType {
        self.workflow_type
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn set_name(&mut self, name: &str) {
        self.name.clear();
        self.name.push_str(name);
    }
}

new_key_type! {
    pub struct AssetKey;
}

#[bulkderive(Persist!)]
pub(in crate::core) struct Ticket(AssetKey);
impl Ticket {
    pub fn key(&self) -> AssetKey {
        self.0
    }
}
