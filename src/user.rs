use std::fs;

const USER_DATABASE_PATH: &str = "/etc/passwd";
const USER_GROUP_PATH: &str = "/etc/group";

fn get_name_from_db(id: u32, db_string: String) -> String {
    let mut name = String::new();
    for line in db_string.lines() {
        if line.contains(format!(":{}:", id).as_str()) {
            for ch in line.chars() {
                if ch == ':' {
                    return name;
                }
                name.push(ch);
            }
        }
    }
    return name;
}

pub fn get_by_uid(uid: u32) -> Result<String, String> {
    return match fs::read_to_string(USER_DATABASE_PATH) {
        Ok(user_db) => Ok(get_name_from_db(uid, user_db)),
        Err(err) => Err(format!("Error getting user name for uid {}: {}", uid, err))
    }
}

pub fn group_by_gid(gid: u32) -> Result<String, String> {
    return match fs::read_to_string(USER_GROUP_PATH) {
        Ok(group_db) => Ok(get_name_from_db(gid, group_db)),
        Err(err) => Err(format!("Error getting group name for gid {}: {}", gid, err))
    }
}