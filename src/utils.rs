use std::time::Duration;

pub fn get_end(last_timestamp: &str) -> Option<String> {
    let segs = last_timestamp.split(":").collect::<Vec<&str>>();

    let hrs = {
        if segs.len() == 2 {
            0_u8
        } else {
            segs.get(segs.len() - 3)
                .unwrap_or(&"0")
                .parse::<u8>()
                .ok()?
        }
    };

    let mins = segs.get(segs.len() - 2)?.parse::<u8>().ok()?;
    let secs = segs.last()?.parse::<u8>().ok()?;

    let added_secs = secs + 6;

    if added_secs <= 60 {
        Some(format!("{:02}:{:02}:{:02}", hrs, mins, added_secs))
    } else {
        let remainder = added_secs - 60;

        let mins = (Duration::from_mins(mins.into()) + Duration::from_mins(1)).as_secs() / 60;

        Some(format!("{:02}:{:02}:{:02}", hrs, mins, remainder))
    }
}

pub fn format(timestamp: &str) -> Option<String> {
    let segs = timestamp.split(":").collect::<Vec<&str>>();

    let hrs = {
        if segs.len() == 2 {
            0_u8
        } else {
            segs.get(segs.len() - 3)
                .unwrap_or(&"0")
                .parse::<u8>()
                .ok()?
        }
    };

    let mins = segs.get(segs.len() - 2)?.parse::<u8>().ok()?;
    let secs = segs.last()?.parse::<u8>().ok()?;

    Some(format!("{:02}:{:02}:{:02}.000", hrs, mins, secs))
}
