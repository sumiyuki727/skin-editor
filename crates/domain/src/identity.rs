use core::num::NonZeroU128;

macro_rules! identity {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU128);

        impl $name {
            pub const fn new(value: u128) -> Option<Self> {
                match NonZeroU128::new(value) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            pub const fn get(self) -> u128 {
                self.0.get()
            }
        }
    };
}

identity!(AssetId);
identity!(DocumentId);
identity!(LayerId);
identity!(ComponentId);
identity!(OutputId);
identity!(AudioComponentId);
