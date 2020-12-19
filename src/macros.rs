pub(crate) const fn check_unique_ids(ids: &[u32]) -> bool {
    let mut i = 0;
    while i < ids.len() {
        let mut count = 0;
        let mut j = 0;
        while j < ids.len() {
            if ids[i] == ids[j] {
                count += 1;
            }
            j += 1;
        }
        if count > 1 {
            return false;
        }
        i += 1;
    }
    return true;
}

#[macro_export]
macro_rules! serde {
    (
        $(
            #[$attrib:meta]
        )*
        $struct_vis:vis struct $name:ident {
            $(
                $field_vis:vis $field:ident: $type:ty = $id:literal,
            )+
        }
    ) => {
        $(
            #[$attrib]
        )*
        $struct_vis struct $name {
            $(
                $field_vis $field: $type,
            )+
        }

        impl $name {
            #[allow(unused)]
            const fn check_unique_ids() -> bool {
                $crate::macros::check_unique_ids(&[$($id),+])
            }
        }

        impl Serialize for $name {
            fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), SerializeError> {
                const_assert!($name::check_unique_ids());
                serializer.start_struct()?;
                $(
                    serializer.serialize_struct_field($id, stringify!($field), &self.$field)?;
                )+
                serializer.end_struct()?;
                Ok(())
            }
        }
    };
    (
        $(
            #[$attrib:meta]
        )*
        $enum_vis:vis enum $name:ident {
            $(
                $variant:ident = $id:literal,
            )+
        }
    ) => {
        $(
            #[$attrib]
        )*
        $enum_vis enum $name {
            $(
                $variant,
            )+
        }

        impl $name {
            #[allow(unused)]
            const fn check_unique_ids() -> bool {
                $crate::macros::check_unique_ids(&[$($id),+])
            }
        }

        impl Serialize for $name {
            fn serialize(&self, serializer: &mut dyn Serializer) -> Result<(), SerializeError> {
                const_assert!($name::check_unique_ids());
                match *self {
                    $(
                        Self::$variant => serializer.serialize_enum($id, stringify!($variant))?,
                    )+
                }
                Ok(())
            }
        }
    }
}
