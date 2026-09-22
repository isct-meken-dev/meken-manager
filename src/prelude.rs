pub use macro_rules_attribute::apply as bulkderive;

#[allow(non_snake_case)]
macro_rules! Persist {
    ($item:item) => {
        #[derive(::core::fmt::Debug, ::serde::Serialize, ::serde::Deserialize)]
        $item
    };
}
pub(crate) use Persist;

macro_rules! errors {
    (
        $vis_err:vis enum $Error:ident {
            $( $Transparent:ident => $Target:path ),* $(,)?
        } {
            $( $Variant:ident
            $( ( $($t_field:ty),* $(,)? ) )?
            $( { $( $s_field:ident : $s_ty:ty ),* $(,)? } )?
            => $msg:literal ),* $(,)?
        }
        $vis_res:vis type $Result:ident<$T:ident>;
    ) => {
        #[derive(::thiserror::Error, ::core::fmt::Debug)]
        $vis_err enum $Error {
            $(
                #[error(transparent)]
                $Transparent(#[from] $Target),
            )*
            $(
                #[error($msg)]
                $Variant
                $( ( $($t_field),* ) )?
                $( { $( $s_field : $s_ty ),* } )?,
            )*
        }
        $vis_res type $Result<$T> = ::core::result::Result<$T, $Error>;
    };
}
pub(crate) use errors;
