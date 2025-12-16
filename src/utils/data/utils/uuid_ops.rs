use uuid::Uuid;

pub struct UuidOps;

impl UuidOps {
    pub fn generate_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn generate_short_id() -> String {
        Uuid::new_v4().to_string()[..8].to_string()
    }
}
