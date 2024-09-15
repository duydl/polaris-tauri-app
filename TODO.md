STEP

1. Change from diesel to Rusqlite then perhaps to sqlx. 
    Search for `use diesel` and `use crate::db`. diesel code is invasive. would take quite a time
    Finish understand barely how db is used with service. Convert ddns.rs service to use sqlite. currently while db is updated, the app is interrupted
2. Remove the need for authen.
3. Change away from actix (to tokio like)
4. Remove other dependencies / or understand them. Simplify everything
5. Remove swapper docs api, ui, 
6. Make both FE and BE share the same localhost host
7. Add ability to listen to additional server
8. New GUI. TUI GUI and GUI web gui. 
9. Compile to mobile
10. Open epub