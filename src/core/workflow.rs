use crate::prelude::*;
use macros::*;

errors! {
    pub enum WorkflowError {} {
        TypeMismatch => "type mismatch",
    }
    pub type WorkflowResult<T>;
}

workflow_enum! {
    #[bulkderive(Persist!)]
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub enum WorkflowType {
        Layout,
        KeyAnimation,
        InBetween,
        Painting,
    }
    #[bulkderive(Persist!)]
    #[derive(Clone, Eq, PartialEq)]
    pub enum Workflow;
    impl Workflow {
        pub fn new(ty: WorkflowType) -> Self;
        pub fn get_type(&self) -> WorkflowType;
    }
}

#[bulkderive(Persist!)]
#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub enum LayoutState {
    #[default]
    NotStarted,
    Completed,
}
#[bulkderive(Persist!)]
#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub enum KeyAnimationState {
    #[default]
    NotStarted,
    Completed,
}
#[bulkderive(Persist!)]
#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub enum InBetweenState {
    #[default]
    NotStarted,
    Completed,
}
#[bulkderive(Persist!)]
#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub enum PaintingState {
    #[default]
    NotStarted,
    Completed,
}

mod macros {
    macro_rules! workflow_enum {
        (
            $(#[$attr_ty:meta])*
            $vis_ty:vis enum $WorkflowType:ident {
                $($Variant:ident),* $(,)?
            }
            $(#[$attr:meta])*
            $vis:vis enum $Workflow:ident;
            impl $Workflow_:ident {
                $vis_ty_wf:vis fn $ty_wf:ident($ty_wf_arg:ident: $WorkflowType_:ty) -> Self;
                $vis_wf_ty:vis fn $wf_ty:ident(&self) -> $WorkflowType__:ty;
            }
        ) => { paste::paste! {
            $(#[$attr_ty])*
            $vis_ty enum $WorkflowType {
                $($Variant),*
            }
            $(#[$attr])*
            $vis enum $Workflow {
                $($Variant([<$Variant State>])),*
            }
            impl $Workflow_ {
                $vis_ty_wf fn $ty_wf($ty_wf_arg: $WorkflowType_) -> Self {
                    match $ty_wf_arg {
                        $(
                            $WorkflowType_::$Variant
                            => $Workflow_::$Variant([<$Variant State>]::default())
                        ),*
                    }
                }
                $vis_wf_ty fn $wf_ty(&self) -> $WorkflowType__ {
                    match self {
                        $(
                            $Workflow_::$Variant(_) => $WorkflowType__::$Variant
                        ),*
                    }
                }
            }
        }};
    }
    pub(super) use workflow_enum;
}
