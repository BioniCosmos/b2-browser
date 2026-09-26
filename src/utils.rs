pub fn dir(path: &str) -> String {
    let dir = &path[..path.rfind('/').unwrap()];
    if dir.is_empty() { "/" } else { dir }.to_owned()
}
