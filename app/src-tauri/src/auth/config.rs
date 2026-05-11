pub const DESKTOP_CLIENT_ID: &str = match option_env!("MLMAIL_GOOGLE_DESKTOP_CLIENT_ID") {
    Some(v) => v,
    None => "REPLACE_ME_DESKTOP_CLIENT_ID",
};

pub const ANDROID_CLIENT_ID: &str = match option_env!("MLMAIL_GOOGLE_ANDROID_CLIENT_ID") {
    Some(v) => v,
    None => "REPLACE_ME_ANDROID_CLIENT_ID",
};

pub const ANDROID_WEB_CLIENT_ID: &str = match option_env!("MLMAIL_GOOGLE_ANDROID_WEB_CLIENT_ID") {
    Some(v) => v,
    None => "REPLACE_ME_ANDROID_WEB_CLIENT_ID",
};
