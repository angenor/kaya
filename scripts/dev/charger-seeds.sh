#!/usr/bin/env bash
#
# Charge les données de démonstration — **une commande, idempotente**.
#
#   bash scripts/dev/charger-seeds.sh                     ajoute ce qui manque
#   bash scripts/dev/charger-seeds.sh --remettre-a-neuf   efface d'abord les séjours d'essai
#
# ## Pourquoi ce script existe alors que le binaire suffirait
#
# `cargo run -p kaya-api --bin seeds` fonctionne, et personne ne le tape correctement du premier
# coup : il faut être dans `backend/`, avoir chargé `.env`, et savoir que `KAYA_SEEDS_MOT_DE_PASSE`
# est exigé **avant toute connexion**. Le quickstart du cycle 006 promet « une commande,
# idempotente » ; ce fichier est cette commande.
#
# ## ★ `--remettre-a-neuf` — pourquoi c'est un ordre SQL et non une option du binaire
#
# Les seeds **ajoutent**, ils n'effacent jamais : c'est ce qui les rend sûrs à relancer. Mais un
# parcours de démonstration — ou la porte P-22 — **consomme des chambres**, et la promesse de
# FR-105 (« rechargeable autant de fois que voulu, avec le même résultat ») devient fausse dès la
# deuxième exécution : la catégorie se remplit, et l'écran affiche « toutes les chambres sont
# prises ».
#
# La remise à neuf a d'abord été écrite **dans le binaire**, sous le rôle applicatif. Elle a été
# refusée par la base :
#
#     permission denied for table ligne_sejour
#
# **Et le modèle de privilèges avait raison.** `kaya_app` ne reçoit aucun `DELETE` sur la note d'un
# séjour ni sur ses lignes : une note est un registre d'exploitation, une correction y est une
# **ligne d'ajustement**, jamais une suppression. Accorder le `DELETE` pour faire tenir un script
# de développement aurait ouvert dans le produit un chemin que le produit refuse — pour la commodité
# d'un poste de travail.
#
# La remise à neuf est donc ce qu'elle est vraiment : une opération d'**administration de base**,
# faite sous le rôle propriétaire, hors de l'application, et jamais dans une image de production.
#
# ⚠️ **Le grand livre n'est jamais purgé** (porte P-05b) : les événements des séjours effacés
# restent, et c'est correct — ils disent ce qui s'est réellement passé sur cette base.
#
# ⚠️ **Un séjour CLOS non seedé n'est pas effacé** : son constat de taxe est immuable par privilège
# (`GRANT SELECT, INSERT` seuls). Un constat figé ne se défige pas, même en développement.
#
# ## Ce qu'il ne fait PAS
#
# Il ne démarre ni la base, ni l'API, et il n'applique aucune migration. Ce sont
# `infra/compose.yml` et `scripts/dev/preparer-base.sh`, dans cet ordre — un script qui ferait tout
# masquerait laquelle des trois étapes a échoué.
#
# ## Idempotence
#
# Les seeds sont rejouables par construction : identifiants littéraux et `ON CONFLICT DO NOTHING`.
# `backend/tests/seeds_rejouables.rs` le vérifie sur **trois** exécutions — deux attrapent la
# duplication franche, trois attrapent le seed qui *met à jour* ce qu'il trouve.

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

if [[ -f backend/.env ]]; then
    set -a
    # shellcheck disable=SC1091
    source backend/.env
    set +a
fi

if [[ -z "${KAYA_SEEDS_MOT_DE_PASSE:-}" ]]; then
    echo "✗ KAYA_SEEDS_MOT_DE_PASSE absent." >&2
    echo "  C'est la variable dont le binaire de seeds se sert pour créer les comptes de" >&2
    echo "  démonstration ; aucun mot de passe n'est écrit dans le dépôt. Voir" >&2
    echo "  backend/.env.example, puis renseigner backend/.env." >&2
    exit 1
fi

# Les deux tenants de démonstration et leurs quatre séjours, **littéralement** — les mêmes
# identifiants que `backend/api/src/bin/seeds.rs`. Ils sont recopiés ici et c'est le prix de
# l'opération : elle vit hors du binaire pour la raison écrite en tête.
TENANTS=(
    '0198c4a0-0000-7000-8000-000000000001'
    '0198c4a0-0000-7000-8000-000000000011'
)
SEJOURS_SEEDES="'0198c4a0-0000-7000-8000-000000000411',
                '0198c4a0-0000-7000-8000-000000000412',
                '0198c4a0-0000-7000-8000-000000000413',
                '0198c4a0-0000-7000-8000-000000000461'"

if [[ "${1:-}" == "--remettre-a-neuf" ]]; then
    if [[ "${KAYA_ENVIRONNEMENT:-}" == "production" ]]; then
        echo "✗ refus : --remettre-a-neuf efface des données, et KAYA_ENVIRONNEMENT=production." >&2
        exit 1
    fi
    if [[ -z "${DATABASE_URL:-}" ]]; then
        echo "✗ DATABASE_URL absent : la remise à neuf se fait sous le rôle propriétaire." >&2
        exit 1
    fi

    echo "── Remise à neuf — les séjours d'essai ───────────────────────────────────────"
    for tenant in "${TENANTS[@]}"; do
        psql "$DATABASE_URL" --quiet --no-psqlrc -v ON_ERROR_STOP=1 <<SQL
SET app.current_tenant = '${tenant}';
-- L'ordre suit les clés étrangères, du plus dépendant au moins dépendant.
DELETE FROM hebergement.ligne_sejour l USING hebergement.note_sejour n
  WHERE l.note_id = n.id AND n.sejour_id NOT IN (${SEJOURS_SEEDES});
DELETE FROM hebergement.note_sejour   WHERE sejour_id NOT IN (${SEJOURS_SEEDES});
DELETE FROM hebergement.fiche_police  WHERE sejour_id NOT IN (${SEJOURS_SEEDES});
DELETE FROM hebergement.accompagnant  WHERE sejour_id NOT IN (${SEJOURS_SEEDES});
DELETE FROM hebergement.occupation    WHERE sejour_id IS NOT NULL
                                        AND sejour_id NOT IN (${SEJOURS_SEEDES});
DELETE FROM hebergement.sejour        WHERE id NOT IN (${SEJOURS_SEEDES});
SQL
    done
    echo "  ✓ séjours d'essai effacés — le grand livre, lui, est intact (P-05b)"
fi

echo "── Chargement des données de démonstration ───────────────────────────────────"
(cd backend && cargo run --quiet -p kaya-api --bin seeds)

echo
echo "✓ Deux tenants, trois comptes, 21 unités, 14 fiches clientes, 4 séjours."
echo "  Rejouable : relancer cette commande laisse exactement le même état."
