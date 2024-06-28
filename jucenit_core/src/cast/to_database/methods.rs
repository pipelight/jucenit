use strum::EnumIter;

#[derive(EnumIter)]
pub enum MatchCategory {
    Managed,
    Unmanaged,
    HttpChallenge,
    TlsAlpnChallenge,
}
