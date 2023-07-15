pub mod constant {
    use chrono::Duration;
    use once_cell::sync::Lazy;

    pub static ACCESS_TOKEN_DUR: Lazy<Duration> = Lazy::new(||{
        chrono::Duration::minutes(30)
    });
    pub static REFRESH_TOKEN_DUR: Lazy<Duration> = Lazy::new(||{
        chrono::Duration::days(3)
    });
    
}
