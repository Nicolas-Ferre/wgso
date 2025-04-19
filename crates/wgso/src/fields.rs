use crate::type_::Type;
use crate::Program;

#[derive(Debug)]
pub(crate) struct StorageField<'a> {
    pub(crate) buffer_name: String,
    pub(crate) type_: &'a Type,
}

impl<'a> StorageField<'a> {
    pub(crate) fn parse(program: &'a Program, path: &str) -> Option<Self> {
        let segments: Vec<_> = path.split('.').collect();
        let storage_type = &program.resources.storages.get(segments[0])?;
        let field_type = storage_type.field_name_type(&segments[1..])?;
        Some(Self {
            buffer_name: segments[0].into(),
            type_: field_type,
        })
    }
}
