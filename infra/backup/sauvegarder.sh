#!/usr/bin/env bash
#
# Sauvegarde quotidienne — **chiffrée avant transfert, déposée sur un hôte distinct**.
#
# ## La ligne qui compte (R-13, FR-060)
#
# Garage tourne sur **le même VPS que la base**. Un attaquant qui obtient le serveur obtient les
# deux, et un rançongiciel qui chiffre le disque chiffre la sauvegarde avec. Garage reçoit donc
# une **copie de travail** — celle qu'on restaure pour un incident ordinaire — et **ne porte
# jamais l'immutabilité**.
#
# La copie opposable part vers un **stockage objet tiers, sur un hôte distinct**, avec
# verrouillage d'objet et rétention verrouillée. C'est la seule ligne de ce cycle qui protège
# contre une compromission plutôt que contre une panne.
#
# Une sauvegarde présente uniquement dans le stockage objet local **ne satisfait pas FR-060** :
# les deux tomberaient ensemble.
#
# ## Chiffrement avant transfert, jamais après
#
# Le chiffrement se fait sur le serveur, avant que le moindre octet ne parte. Confier le
# chiffrement au fournisseur de stockage reviendrait à lui confier la clé — et les dumps portent
# les pièces d'identité des clients du pilote.
#
# `age` est l'outil retenu : une clé publique suffit pour chiffrer, la clé privée n'a donc jamais
# besoin d'exister sur le serveur de production. Un serveur compromis ne permet pas de relire les
# sauvegardes qu'il a produites.
#
# ## Usage
#
#     infra/backup/sauvegarder.sh
#
# Variables attendues (voir `infra/backup/README.md`) :
#
#     PGHOST PGPORT PGDATABASE PGUSER PGPASSWORD
#     KAYA_BACKUP_AGE_RECIPIENT     clé publique age — chiffrement
#     KAYA_BACKUP_S3_TIERS_BUCKET   compartiment TIERS, hôte distinct, verrouillage d'objet
#     KAYA_BACKUP_S3_TIERS_ENDPOINT
#     KAYA_BACKUP_S3_LOCAL_BUCKET   Garage — copie de travail, JAMAIS l'immutabilité

set -euo pipefail

horodatage="$(date -u +%Y%m%dT%H%M%SZ)"
base="${PGDATABASE:-kaya}"
repertoire_travail="$(mktemp -d)"
trap 'rm -rf "$repertoire_travail"' EXIT

nom="kaya-${base}-${horodatage}.dump"
chemin_dump="${repertoire_travail}/${nom}"
chemin_chiffre="${chemin_dump}.age"

exiger() {
    if [[ -z "${!1:-}" ]]; then
        echo "✗ variable requise absente : $1" >&2
        exit 1
    fi
}

exiger KAYA_BACKUP_AGE_RECIPIENT
exiger KAYA_BACKUP_S3_TIERS_BUCKET
exiger KAYA_BACKUP_S3_TIERS_ENDPOINT

echo "── Dump ──────────────────────────────────────────────────────────────────────"
# `--format=custom` : compressé, et surtout **restaurable table par table**. Un dump SQL brut
# impose de tout rejouer, ce qui rend inutilisable la restauration partielle — celle dont on a
# besoin quand une seule table a été corrompue.
pg_dump \
    --format=custom \
    --compress=9 \
    --no-owner \
    --no-privileges \
    --file="$chemin_dump" \
    "$base"

octets=$(wc -c < "$chemin_dump" | tr -d ' ')
echo "  ${nom} — ${octets} octets"

if [[ "$octets" -lt 1024 ]]; then
    # Un dump quasi vide est presque toujours le signe d'une erreur de connexion silencieuse.
    # Le détecter ici évite de découvrir six mois plus tard que la rétention ne contient rien.
    echo "✗ dump suspect (< 1 Kio) — sauvegarde interrompue" >&2
    exit 1
fi

echo "── Chiffrement (avant tout transfert) ────────────────────────────────────────"
age --encrypt --recipient "$KAYA_BACKUP_AGE_RECIPIENT" --output "$chemin_chiffre" "$chemin_dump"
rm -f "$chemin_dump"
echo "  ${nom}.age"

echo "── Dépôt 1/2 — stockage TIERS, hôte distinct, verrouillé ─────────────────────"
# `--endpoint-url` pointe le fournisseur tiers, jamais Garage. Le verrouillage d'objet et la
# rétention verrouillée sont posés **sur le compartiment**, à sa création : les poser objet par
# objet laisserait un chemin pour les omettre.
aws s3 cp "$chemin_chiffre" \
    "s3://${KAYA_BACKUP_S3_TIERS_BUCKET}/quotidien/${nom}.age" \
    --endpoint-url "$KAYA_BACKUP_S3_TIERS_ENDPOINT"
echo "  déposé sur le tiers"

if [[ -n "${KAYA_BACKUP_S3_LOCAL_BUCKET:-}" ]]; then
    echo "── Dépôt 2/2 — copie de travail sur Garage (NON immuable) ────────────────────"
    aws s3 cp "$chemin_chiffre" \
        "s3://${KAYA_BACKUP_S3_LOCAL_BUCKET}/quotidien/${nom}.age" \
        --endpoint-url "${S3_ENDPOINT:-http://localhost:3900}"
    echo "  copie locale déposée — restauration rapide d'un incident ordinaire"
fi

echo
echo "Sauvegarde ${nom}.age terminée."
echo "Rappel : la copie qui protège d'une compromission est celle du TIERS. La copie Garage"
echo "tombe avec le serveur qu'elle est censée sauver."
