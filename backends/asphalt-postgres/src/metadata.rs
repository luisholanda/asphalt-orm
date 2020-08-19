use parking_lot::Mutex;
use std::collections::HashMap;
use tokio_postgres::types::Type;

#[derive(Default)]
pub struct MetadataLookup {
    typ_cache: Mutex<HashMap<(String, String), Type>>,
}

impl MetadataLookup {
    pub fn get_type_metadata_for(&self, type_name: String, schema_name: String) -> Option<Type> {
        self.typ_cache
            .lock()
            .get(&(type_name, schema_name))
            .cloned()
    }

    pub(crate) fn register_type_metadata(&self, typ: Type) {
        let typ_name = typ.name().to_string();
        let sch_name = typ.schema().to_string();

        self.typ_cache.lock().insert((typ_name, sch_name), typ);
    }
}
