//! Compact numeric IDs for catalog entries.
//!
//! Data files reference entries by readable keys (`roheisen`, `DEU`). When the catalog
//! is built, every key gets a dense index so the simulation can use plain vectors.

/// Common behaviour of all catalog IDs.
pub trait Id: Copy + Ord {
    fn from_index(index: usize) -> Self;
    fn index(self) -> usize;
}

macro_rules! define_id {
    ($($(#[$meta:meta])* $name:ident),* $(,)?) => {$(
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        pub struct $name(u16);

        impl Id for $name {
            fn from_index(index: usize) -> Self {
                Self(u16::try_from(index).expect(concat!("too many entries for ", stringify!($name))))
            }

            fn index(self) -> usize {
                usize::from(self.0)
            }
        }
    )*};
}

define_id!(
    UnitId,
    ContinentId,
    BranchId,
    GoodsGroupId,
    TransportClassId,
    QualificationId,
    SpecializationId,
    /// A qualification, optionally combined with a specialization (`fachkraft.metall`).
    LaborGroupId,
    CountryId,
    ProductId,
    FacilityId,
    RecipeId,
    TechnologyId,
    DepositId,
);
