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

        impl $crate::ser::Serialize for $name {
            fn serialize(&self, serializer: &mut dyn $crate::ser::Serializer) -> Result<(), $crate::ser::SerializeError> {
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

        impl $crate::ser::Serialize for $name {
            fn serialize(&self, serializer: &mut dyn $crate::ser::Serializer) -> Result<(), $crate::ser::SerializeError> {
                const_assert!($name::check_unique_ids());
                match *self {
                    $(
                        Self::$variant => serializer.serialize_enum($id, stringify!($variant))?,
                    )+
                }
                Ok(())
            }
        }

        impl $crate::Deserialize for $name {
            fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn $crate::Visitor {
                impl $crate::Visitor for $crate::Place<$name> {
                    fn visit_str(&mut self, value: &str) -> Result<(), $crate::DeserializeError> {
                        let (id, name) = if let Some(colon_index) = value.find(':') {
                            let id = value[..colon_index].parse::<i64>().ok();
                            let name = &value[(colon_index + 1)..];
                            (id, name)
                        }
                        else {
                            (None, value)
                        };

                        if let Some(id) = id {
                            let variant = match id {
                                $(
                                    $id => Some($name::$variant),
                                )+
                                _ => None,
                            };

                            if let Some(variant) = variant {
                                self.out.replace(variant);
                                return Ok(());
                            }
                        }

                        let variant = match name {
                            $(
                                stringify!($variant) => Some($name::$variant),
                            )+
                            _ => None,
                        };

                        if let Some(variant) = variant {
                            self.out.replace(variant);
                            return Ok(());
                        }
                        else {
                            return Err($crate::DeserializeError::UnknownEnumVariant);
                        }
                    }

                    fn visit_signed(&mut self, value: i64) -> Result<(), $crate::DeserializeError> {
                        if value < 0 {
                            Err($crate::DeserializeError::UnknownEnumVariant)
                        }
                        else {
                            self.visit_unsigned(value as u64)
                        }
                    }

                    fn visit_unsigned(&mut self, value: u64) -> Result<(), $crate::DeserializeError> {
                        let variant = match value {
                            $(
                                $id => Some($name::$variant),
                            )+
                            _ => None,
                        };

                        if let Some(variant) = variant {
                            self.out.replace(variant);
                            return Ok(());
                        }
                        else {
                            return Err($crate::DeserializeError::UnknownEnumVariant);
                        }
                    }
                }
                return Place::new(out);
            }
        }
    }
}
