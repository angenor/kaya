#!/usr/bin/env bash
#
# Prépare la base de développement : migrations, puis mots de passe locaux.
#
# **Pourquoi les mots de passe ne sont pas dans la migration.** Un secret écrit dans un fichier
# de migration est un secret dans l'historique Git, en clair, pour toujours — et une migration
# appliquée ne se modifie jamais (principe I(b)), donc l'erreur serait définitive. La migration
# 0001 crée donc les trois rôles **sans mot de passe** ; ce script pose ceux du poste de
# développement, la CI fait de même avec les siens, et la production les tient hors du dépôt.
#
# Usage :
#   scripts/dev/preparer-base.sh              # migrations + mots de passe de développement
#   scripts/dev/preparer-base.sh --recreer    # repart d'une base vide

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

HOTE="${PGHOST:-localhost}"
PORT="${POSTGRES_PORT:-5433}"
BASE="${PGDATABASE:-kaya}"
MDP_DEV="${KAYA_MDP_DEV:-motdepasse_dev}"

export PGPASSWORD="${POSTGRES_PASSWORD:-motdepasse_dev}"

psql_super() { psql -h "$HOTE" -p "$PORT" -U postgres -d "$BASE" -v ON_ERROR_STOP=1 "$@"; }

echo "── Attente de la base ────────────────────────────────────────────────────────"
for _ in $(seq 1 60); do
    if pg_isready -h "$HOTE" -p "$PORT" -U postgres -d "$BASE" >/dev/null 2>&1; then break; fi
    sleep 1
done
pg_isready -h "$HOTE" -p "$PORT" -U postgres -d "$BASE"

if [[ "${1:-}" == "--recreer" ]]; then
    echo "── Remise à zéro ─────────────────────────────────────────────────────────────"
    psql_super -c "DROP SCHEMA IF EXISTS etablissements, synchronisation, fiscalite, kaya_migrations CASCADE;"
fi

echo "── Rôle propriétaire ─────────────────────────────────────────────────────────"
# Idempotent : le script d'initialisation du conteneur l'a déjà créé au premier démarrage, mais
# ce script doit aussi fonctionner sur une base préexistante.
psql_super <<SQL
DO \$\$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'kaya_owner') THEN
        CREATE ROLE kaya_owner LOGIN NOSUPERUSER NOBYPASSRLS CREATEROLE;
    END IF;
END
\$\$;
ALTER ROLE kaya_owner PASSWORD '${MDP_DEV}';
ALTER DATABASE ${BASE} OWNER TO kaya_owner;
GRANT ALL ON SCHEMA public TO kaya_owner;
SQL

echo "── Migrations (sous kaya_owner, R-12) ────────────────────────────────────────"
export DATABASE_URL="postgres://kaya_owner:${MDP_DEV}@${HOTE}:${PORT}/${BASE}"
# Exécuté depuis `backend/api` : c'est là que vit `sqlx.toml`, et sqlx-cli le cherche dans le
# répertoire courant. Le lancer depuis `backend/` ferait tenir au CLI et à la macro
# `sqlx::migrate!()` deux tables de suivi différentes — voir l'en-tête de `backend/api/sqlx.toml`.
(cd backend/api && cargo sqlx migrate run --source ../migrations)

echo "── Mots de passe applicatifs ─────────────────────────────────────────────────"
PGPASSWORD="$MDP_DEV" psql -h "$HOTE" -p "$PORT" -U kaya_owner -d "$BASE" -v ON_ERROR_STOP=1 <<SQL
ALTER ROLE kaya_app            PASSWORD '${MDP_DEV}';
ALTER ROLE kaya_ledger_reader  PASSWORD '${MDP_DEV}';
SQL

echo
echo "Base prête. Chaînes de connexion du poste de développement :"
echo "  DATABASE_URL          postgres://kaya_owner:***@${HOTE}:${PORT}/${BASE}"
echo "  DATABASE_URL_APP      postgres://kaya_app:***@${HOTE}:${PORT}/${BASE}"
echo "  DATABASE_URL_LEDGER   postgres://kaya_ledger_reader:***@${HOTE}:${PORT}/${BASE}"
