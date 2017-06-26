
#[derive(Serialize, Deserialize)]
struct ProfileDef {
    name: String,
    pattern: String,
    modes: Vec<ModeDef>,
}

#[derive(Serialize, Deserialize)]
struct ModeDef {
    name: String,
    devices: Vec<DeviceDef>,
}

#[derive(Serialize, Deserialize)]
struct DeviceDef {
    device: String,
    single: Vec<SingleDef>,
}

#[derive(Serialize, Deserialize)]
struct SingleDef {
    key: String,
    cmd: String,
}



