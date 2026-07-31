#!/usr/bin/env bash
#
# Prépare le stockage objet de développement — cluster, clé d'accès, compartiment.
#
# ## Pourquoi ce script existe
#
# Un conteneur Garage qui démarre est **sain sans être utilisable** : son nœud n'a aucun rôle
# assigné, aucune clé d'accès n'existe, aucun compartiment non plus. `docker compose ps` le
# rapporte pourtant `healthy`, et c'est exact — le service répond.
#
# La sonde `/health`, elle, tente un appel S3 réel et rapporte `degrade`. C'est précisément la
# différence que la sonde sert à faire, et elle l'a faite ici : sans ce script, l'amorçage de
# SC-001 s'arrêtait à deux dépendances opérationnelles sur trois, sans que rien n'explique la
# troisième.
#
# ## Ce que le script ne fait pas
#
# Il ne touche **jamais** à la production. Le cluster de production a plusieurs nœuds, une
# capacité réelle et des clés au coffre ; celui-ci a un nœud, un gigaoctet et des clés de
# développement affichées en clair à la fin.
#
#     scripts/dev/preparer-stockage.sh

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

CONTENEUR="${KAYA_GARAGE_CONTENEUR:-kaya-objets}"
COMPARTIMENT="${S3_BUCKET:-kaya-dev}"
NOM_CLE="${KAYA_GARAGE_CLE:-kaya-dev}"

garage() { docker exec "$CONTENEUR" /garage "$@" 2>/dev/null; }

echo "── Attente du nœud ───────────────────────────────────────────────────────────"
for _ in $(seq 1 30); do
    if garage status >/dev/null 2>&1; then break; fi
    sleep 2
done

noeud="$(garage status | awk '/^[0-9a-f]{16}[[:space:]]/ { print $1; exit }')"
if [[ -z "$noeud" ]]; then
    echo "✗ aucun nœud Garage joignable dans le conteneur « $CONTENEUR »" >&2
    exit 1
fi
echo "  nœud : $noeud"

echo "── Attribution du rôle ───────────────────────────────────────────────────────"
# Un nœud sans rôle ne stocke rien. Un seul nœud, une seule zone : c'est un environnement de
# développement, et `replication_factor = 1` de `infra/garage/garage.toml` le dit déjà.
if garage layout show | grep -q "NO ROLE ASSIGNED" || ! garage layout show | grep -q "$noeud"; then
    garage layout assign -z dev -c 1G "$noeud" >/dev/null
    # Le numéro de version à appliquer est celui que Garage annonce ; le deviner ferait échouer
    # la commande sur une installation déjà partiellement configurée.
    version="$(garage layout show | awk '/apply --version/ { print $NF; exit }')"
    garage layout apply --version "${version:-1}" >/dev/null
    echo "  rôle attribué, disposition appliquée"
else
    echo "  · rôle déjà attribué"
fi

echo "── Clé d'accès ───────────────────────────────────────────────────────────────"
if garage key list | grep -q "$NOM_CLE"; then
    echo "  · clé « $NOM_CLE » déjà présente"
else
    garage key create "$NOM_CLE" >/dev/null
    echo "  clé « $NOM_CLE » créée"
fi

info="$(garage key info "$NOM_CLE" --show-secret)"
cle_id="$(awk '/Key ID:/ { print $3 }' <<<"$info")"
cle_secret="$(awk '/Secret key:/ { print $3 }' <<<"$info")"

echo "── Compartiment ──────────────────────────────────────────────────────────────"
if garage bucket list | grep -q "$COMPARTIMENT"; then
    echo "  · compartiment « $COMPARTIMENT » déjà présent"
else
    garage bucket create "$COMPARTIMENT" >/dev/null
    echo "  compartiment « $COMPARTIMENT » créé"
fi
garage bucket allow --read --write "$COMPARTIMENT" --key "$NOM_CLE" >/dev/null
echo "  lecture et écriture accordées à « $NOM_CLE »"

echo
echo "Stockage prêt. À reporter dans backend/.env :"
echo
echo "  S3_ACCESS_KEY=$cle_id"
echo "  S3_SECRET_KEY=$cle_secret"
echo
echo "Ces clés sont de DÉVELOPPEMENT. Celles de production vivent dans le coffre chiffré par"
echo "tenant (principe IX) et ne sont jamais affichées ni commitées."
