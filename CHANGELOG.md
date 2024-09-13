## Finish draft of integrating. App is running.

1. 
```log
error: proc macro panicked
  --> src\db.rs:12:40
   |
12 | const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
   |                                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: message: Failed to receive migrations dir from Some("migrations")
```
Fix with adding `migrations` directory to `src-tauri`

2. Add default test db by modifying `src-tauri\src\options.rs`
Log in with:
`duydl - 123456 at localhost:5050`

3. Add [dev-dependencies] of polaris (for rust-analyzer not complaining)