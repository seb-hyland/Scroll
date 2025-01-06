pub fn serialize(name: &str) -> String {
    name.replace(" ", "_")
}

pub fn deserialize(name: &str) -> String {
    name.replace("_", " ")
}
