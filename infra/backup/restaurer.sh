#!/usr/bin/env bash
#
# Restauration d'une sauvegarde chiffrée.
#
#     infra/backup/restaurer.sh <horodatage> <base_cible>
#
# Exemple :
#
#     infra/backup/restaurer.sh 20260731T031500Z kaya_restauration
#
# **La procédure complète, rédigée pour quelqu'un qui n'a pas écrit le système, est dans
# `infra/backup/README.md`.** Ce script en est l'outil, pas la documentation : le suivre sans
# avoir lu le README fait manquer les deux points qui décident du succès — d'où vient la clé
# privée, et pourquoi on ne restaure jamais par-dessus la base de production.

set -euo pipefail

if [[ $# -lt 2 ]]; then
    echo "usage : $0 <horodatage> <base_cible>" >&2
    echo "        $0 20260731T031500Z kaya_restauration" >&2
    exit 2
fi

horodatage="$1"
base_cible="$2"
base_source="${PGDATABASE:-kaya}"
nom="kaya-${base_source}-${horodatage}.dump"

repertoire_travail="$(mktemp -d)"
trap 'rm -rf "$repertoire_travail"' EXIT

# --- Garde-fou : jamais par-dessus la production ------------------------------------------------
#
# Restaurer sur la base vive écrase les données produites depuis la sauvegarde. La restauration
# se fait TOUJOURS sur une base neuve, qu'on bascule ensuite. Ce refus est ce qui empêche de
# transformer un incident en perte de données, à 3 h du matin, sous pression.
if [[ "$base_cible" == "$base_source" ]]; then
    echo "✗ refus : la base cible est la base de production ($base_source)." >&2
    echo "  Restaurer sur une base NEUVE, puis basculer. Voir infra/backup/README.md §4." >&2
    exit 1
fi

exiger() {
    if [[ -z "${!1:-}" ]]; then
        echo "✗ variable requise absente : $1" >&2
        exit 1
    fi
}
exiger KAYA_BACKUP_AGE_IDENTITY
exiger KAYA_BACKUP_S3_TIERS_BUCKET
exiger KAYA_BACKUP_S3_TIERS_ENDPOINT

debut=$(date +%s)

echo "── 1/4 Récupération depuis le stockage TIERS ─────────────────────────────────"
aws s3 cp \
    "s3://${KAYA_BACKUP_S3_TIERS_BUCKET}/quotidien/${nom}.age" \
    "${repertoire_travail}/${nom}.age" \
    --endpoint-url "$KAYA_BACKUP_S3_TIERS_ENDPOINT"

echo "── 2/4 Déchiffrement ─────────────────────────────────────────────────────────"
# La clé privée ne vit **pas** sur le serveur de production (voir `sauvegarder.sh`) : elle est
# apportée ici, le temps de la restauration. Un serveur compromis ne peut donc pas relire les
# sauvegardes qu'il a produites.
age --decrypt --identity "$KAYA_BACKUP_AGE_IDENTITY" \
    --output "${repertoire_travail}/${nom}" \
    "${repertoire_travail}/${nom}.age"

echo "── 3/4 Création de la base cible ─────────────────────────────────────────────"
createdb "$base_cible"

echo "── 4/4 Restauration ──────────────────────────────────────────────────────────"
# `--no-owner --no-privileges` : le dump ne porte ni propriétaire ni droits (voir
# `sauvegarder.sh`). Les rôles et les privilèges sont **recréés par les migrations**, seule
# source de vérité du schéma (principe I(b)). Restaurer des droits depuis un dump les figerait à
# leur état du jour de la sauvegarde.
pg_restore \
    --dbname="$base_cible" \
    --no-owner \
    --no-privileges \
    --exit-on-error \
    "${repertoire_travail}/${nom}"

duree=$(( $(date +%s) - debut ))

echo
echo "Restauration terminée en ${duree} s dans la base « ${base_cible} »."
echo
echo "À FAIRE ENSUITE, dans cet ordre :"
echo "  1. Amorcer les rôles  : psql -d ${base_cible} -f infra/postgres/init/00-kaya-owner.sql"
echo "  2. Appliquer les migrations : cd backend/api && cargo sqlx migrate run --source ../migrations"
echo "  3. Vérifier            : les tests de infra/backup/README.md §5"
echo "  4. Consigner la durée  : ${duree} s, dans infra/backup/README.md §6"
