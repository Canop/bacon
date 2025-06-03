pub mod build;
pub mod lint;

fn parse_swift_location(location_str: &str) -> Option<(&str, &str, &str)> {
    // splitting from the right to avoid colons in path
    let parts: Vec<&str> = location_str.rsplitn(3, ':').collect();
    if parts.len() == 3 {
        let path = parts[2];
        let line = parts[1];
        let column = parts[0];
        Some((path, line, column)) // path, line, column
    } else {
        None
    }
}
