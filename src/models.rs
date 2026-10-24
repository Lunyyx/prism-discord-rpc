pub struct Instance {
    pub name: String,
    pub minecraft_version: String,
}

pub struct Session {
    pub started_at: u64,
    pub instance: Instance,
}
