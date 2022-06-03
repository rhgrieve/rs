pub enum DateFormat {
    Numeric,
    FullMonth,
    ShortMonth,
}

pub struct SimpleDate {
    year: u64,
    month: u64,
    day: u64,
}

impl SimpleDate {
    // Stolen with great respect from Howard Hinnant :]
    // https://stackoverflow.com/a/32158604
    pub fn from_days(mut days: u64) -> SimpleDate {
        days += 719468;
        let era = (if days > 0 { days } else { days - 146096 } / 146097);
        let doe = days - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = mp + (if mp < 10 { 3 } else { 9 });
        return SimpleDate {
            year: y,
            month: m,
            day: d,
        };
    }

    pub fn year(&self) -> String {
        return self.year.to_string();
    }

    pub fn month(&self) -> String {
        return self.month.to_string();
    }

    pub fn month_display(&self, format: DateFormat) -> String {
        match format {
            DateFormat::Numeric => self.month.to_string(),
            DateFormat::FullMonth => self.month_from_numeric(self.month).unwrap(),
            DateFormat::ShortMonth => {
                let mut month_string = self.month_from_numeric(self.month).unwrap();
                month_string.truncate(3);
                return month_string;
            }
        }
    }

    pub fn day(&self) -> String {
        return self.day.to_string();
    }

    fn month_from_numeric(&self, month_numeric: u64) -> Result<String, String> {
        let selected_month = match month_numeric {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "",
        };

        if selected_month.is_empty() {
            return Err(format!("Invalid month {}. Range is [1,12]", month_numeric));
        }

        Ok(selected_month.to_string())
    }
}
