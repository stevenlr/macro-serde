pub const fn check_unique_ids(ids: &[u32]) -> bool {
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
                $crate::const_assert!($name::check_unique_ids());
                serializer.start_struct()?;
                $(
                    serializer.serialize_struct_field($id, stringify!($field), &self.$field)?;
                )+
                serializer.end_struct()?;
                Ok(())
            }
        }

        impl $crate::de::Deserialize for $name {
            fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn $crate::de::Visitor {
                struct Builder<'a> {
                    deserialize_out_place: &'a mut Option<$name>,
                    $(
                        $field: Option<$type>,
                    )+
                }

                impl<'a> Builder<'a> {
                    fn new(out: &'a mut Option<$name>) -> Self {
                        Self {
                            deserialize_out_place: out,
                            $(
                                $field: None,
                            )+
                        }
                    }
                }

                impl<'a> $crate::de::StructBuilder for Builder<'a> {
                    fn member(&mut self, id: Option<u32>, name: Option<&str>) -> Result<&mut dyn $crate::de::Visitor, $crate::de::DeserializeError> {
                        if let Some(id) = id {
                            match id {
                                $(
                                    $id => return Ok(<$type as $crate::de::Deserialize>::begin_deserialize(&mut self.$field)),
                                )+
                                _ => {},
                            }
                        }

                        if let Some(name) = name {
                            match name {
                                $(
                                    stringify!($field) => return Ok(<$type as $crate::de::Deserialize>::begin_deserialize(&mut self.$field)),
                                )+
                                _ => {},
                            }
                        }

                        return Err($crate::de::DeserializeError::UnknownField);
                    }

                    fn finish(&mut self) -> Result<(), $crate::de::DeserializeError> {
                        let result = $name {
                            $(
                                $field: self.$field.take().ok_or($crate::de::DeserializeError::MissingField(stringify!($field)))?,
                            )+
                        };
                        self.deserialize_out_place.replace(result);
                        Ok(())
                    }
                }

                $crate::make_place_type!(Place);

                impl $crate::de::Visitor for Place<$name> {
                    fn visit_struct<'a>(&'a mut self) -> Result<Box<dyn $crate::de::StructBuilder + 'a>, $crate::de::DeserializeError> {
                        Ok(Box::new(Builder::new(&mut self.out)))
                    }
                }
                return Place::new(out);
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
                $crate::const_assert!($name::check_unique_ids());
                match *self {
                    $(
                        Self::$variant => serializer.serialize_enum($id, stringify!($variant))?,
                    )+
                }
                Ok(())
            }
        }

        impl $crate::de::Deserialize for $name {
            fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn $crate::de::Visitor {
                $crate::make_place_type!(Place);

                impl $crate::de::Visitor for Place<$name> {
                    fn visit_str(&mut self, value: &str) -> Result<(), $crate::de::DeserializeError> {
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
                            return Err($crate::de::DeserializeError::UnknownEnumVariant);
                        }
                    }

                    fn visit_signed(&mut self, value: i64) -> Result<(), $crate::de::DeserializeError> {
                        if value < 0 {
                            Err($crate::de::DeserializeError::UnknownEnumVariant)
                        }
                        else {
                            self.visit_unsigned(value as u64)
                        }
                    }

                    fn visit_unsigned(&mut self, value: u64) -> Result<(), $crate::de::DeserializeError> {
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
                            return Err($crate::de::DeserializeError::UnknownEnumVariant);
                        }
                    }
                }
                return Place::new(out);
            }
        }
    };
    (
        $(
            #[$attrib:meta]
        )*
        $vis:vis union $name:ident {
            $(
                $variant:ident($type:ty) = $id:literal,
            )+
        }
    ) => {
        $(
            #[$attrib]
        )*
        $vis enum $name {
            $(
                $variant($type),
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
                $crate::const_assert!($name::check_unique_ids());
                match self {
                    $(
                        Self::$variant(val) => {
                            serializer.start_struct()?;
                            serializer.serialize_struct_field($id, stringify!($variant), val)?;
                            serializer.end_struct()?;
                        }
                    )+
                }
                Ok(())
            }
        }

        impl $crate::de::Deserialize for $name {
            fn begin_deserialize(out: &mut Option<Self>) -> &mut dyn $crate::de::Visitor {
                $crate::make_place_type!(Place);

                #[allow(non_snake_case)]
                struct Builder<'a> {
                    deserialize_out_place: &'a mut Option<$name>,
                    deserialize_variant_id: Option<u32>,
                    $(
                        $variant: Option<$type>,
                    )+
                }

                impl<'a> Builder<'a> {
                    fn new(out: &'a mut Option<$name>) -> Self {
                        Self {
                            deserialize_out_place: out,
                            deserialize_variant_id: None,
                            $(
                                $variant: None,
                            )+
                        }
                    }
                }

                impl<'a> $crate::de::StructBuilder for Builder<'a> {
                    fn member(
                        &mut self,
                        id: Option<u32>,
                        name: Option<&str>,
                    ) -> Result<&mut dyn $crate::de::Visitor, $crate::de::DeserializeError> {
                        match id {
                            $(
                                Some($id) => {
                                    self.deserialize_variant_id = Some($id);
                                    return Ok(<$type as $crate::de::Deserialize>::begin_deserialize(&mut self.$variant));
                                }
                            )+
                            _ => {},
                        }

                        match name {
                            $(
                                Some(stringify!($variant)) => {
                                    self.deserialize_variant_id = Some($id);
                                    return Ok(<$type as $crate::de::Deserialize>::begin_deserialize(&mut self.$variant));
                                }
                            )+
                            _ => {},
                        }

                        Err($crate::de::DeserializeError::UnknownUnionVariant)
                    }

                    fn finish(&mut self) -> Result<(), $crate::de::DeserializeError> {
                        match self.deserialize_variant_id {
                            $(
                                Some($id) if self.$variant.is_some() => {
                                    self.deserialize_out_place.replace($name::$variant(self.$variant.take().unwrap()));
                                    Ok(())
                                }
                            )+
                            _ => Err($crate::de::DeserializeError::UnknownUnionVariant),
                        }
                    }
                }


                impl $crate::de::Visitor for Place<$name> {
                    fn visit_struct<'a>(&'a mut self) -> Result<Box<dyn StructBuilder + 'a>, DeserializeError> {
                        Ok(Box::new(Builder::new(&mut self.out)))
                    }
                }

                return Place::new(out);
            }
        }
    };
}
