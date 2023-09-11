rm "./database.db"
export DATABASE_URL="sqlite:./database.db"
~/.cargo/bin/sqlx db create
~/.cargo/bin/sqlx migrate run --source database/migrations/
