pub fn is_unique_violation_on_code(e: &sqlx::Error) -> bool {
    let Some(db_err) = e.as_database_error() else {
        return false;
    };

    if !db_err.is_unique_violation() {
        return false;
    }

    matches!(db_err.constraint(), Some("links_code_key"))
}
