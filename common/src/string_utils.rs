use uuid::Uuid;

pub fn rand_id() -> String {
    Uuid::new_v4().to_string()
}
