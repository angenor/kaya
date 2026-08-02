#!/usr/bin/env bash
#
# **Test négatif de la porte P-09** — prouve qu'elle sait échouer.
#
#   scripts/ci/exclusion-negatif.sh
#
# ═════════════════════════════════════════════════════════════════════════════════════════════
#  POURQUOI CE SCRIPT EXISTE
# ═════════════════════════════════════════════════════════════════════════════════════════════
#
#  **Une porte qui n'a jamais échoué n'est pas une porte.** C'est la leçon des quatre portes
#  vertes défectueuses du cycle 001, et la raison pour laquelle la constitution exige un test
#  négatif par porte.
#
#  P-09 vit dans `backend/tests/hebergement_disponibilite.rs`. Ses trois assertions passent — mais
#  passeraient-elles encore si la contrainte d'exclusion disparaissait ? La seule façon de le
#  savoir est de la retirer et de regarder.
#
#  Ce script :
#
#    1. constate que P-09 est verte ;
#    2. **retire** `occupation_sans_chevauchement` de la base de développement ;
#    3. constate que P-09 est ROUGE — et vérifie qu'elle échoue sur les BONNES assertions ;
#    4. **purge les lignes chevauchantes puis remet** la contrainte, quoi qu'il arrive (`trap`) ;
#    5. constate que P-09 est de nouveau verte.
#
#  L'étape 4 est sous `trap` parce qu'une interruption à l'étape 3 laisserait une base de
#  développement sans sa garantie centrale — et personne ne le remarquerait avant longtemps.
#
#  **La purge n'est pas un détail de propreté, c'est ce qui rend la remise possible.** Pendant
#  l'étape 3, le test de concurrence réussit deux attributions chevauchantes — c'est précisément
#  ce qu'il constate. Ces lignes restent en base, et la migration `0025` écrit noir sur blanc
#  qu'une contrainte d'exclusion « ajoutée sur une table peuplée échoue sur les données
#  existantes ». Le premier passage de ce script l'a rencontré pour de vrai.
#
#  **Deux obstacles de plus, découverts au même endroit** : `FORCE ROW LEVEL SECURITY` s'applique
#  aussi au propriétaire, donc la purge ne verrait aucune ligne à supprimer ; et `SET row_security
#  = off` est refusé tant que `FORCE` est posé. L'étape 4 retire donc `FORCE` le temps de la
#  purge, et **le remet**. Un script qui l'oublierait laisserait la table lisible par toute
#  maintenance — la fuite que `FORCE` existe pour fermer.
#
# ═════════════════════════════════════════════════════════════════════════════════════════════
#  CE QUE CE SCRIPT NE FAIT PAS
# ═════════════════════════════════════════════════════════════════════════════════════════════
#
#  Il ne touche **aucune migration**. La contrainte est retirée par `ALTER TABLE` sur la base
#  courante, puis remise à l'identique — la migration `0025` n'est pas modifiée, et la porte P-02
#  reste satisfaite. Sur une base neuve, `0025` la recrée.
#
#  Il ne s'exécute **pas en intégration continue** : il modifie temporairement le schéma, ce qui
#  ferait échouer tout test parallèle. Il se lance à la main, et son résultat se consigne — comme
#  `porte:p22:negatif`.

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

if [[ -f backend/.env ]]; then
    set -a
    # shellcheck disable=SC1091
    source backend/.env
    set +a
fi

if [[ -z "${DATABASE_URL:-}" ]]; then
    echo "  ✗ DATABASE_URL absente. Lancer d'abord :" >&2
    echo "      docker compose -f infra/compose.yml up -d && bash scripts/dev/preparer-base.sh" >&2
    exit 1
fi

CONTRAINTE="occupation_sans_chevauchement"
TABLE="hebergement.occupation"
DEFINITION="EXCLUDE USING gist (unite_id WITH =, periode WITH &&)"

psql_() {
    docker exec -i kaya-db psql -v ON_ERROR_STOP=1 -qtA -U kaya_owner -d kaya "$@"
}

remettre() {
    echo
    echo "── 4/5 — purge des lignes chevauchantes, puis remise en état ─────────────────"

    # `NO FORCE` le temps de la purge : sans lui, le propriétaire lui-même ne voit aucune ligne,
    # et le `DELETE` ne supprimerait rien **sans erreur**. Il est remis juste après.
    purge=$(psql_ <<SQL || true
ALTER TABLE ${TABLE} NO FORCE ROW LEVEL SECURITY;
SET row_security = off;
DELETE FROM ${TABLE} o
 WHERE EXISTS (SELECT 1 FROM ${TABLE} b
                WHERE b.unite_id = o.unite_id AND b.id < o.id AND b.periode && o.periode);
SQL
)
    [[ -n "$purge" ]] && echo "  · lignes chevauchantes purgées"

    if psql_ -c "ALTER TABLE ${TABLE} ADD CONSTRAINT ${CONTRAINTE} ${DEFINITION};" >/dev/null 2>&1
    then
        echo "  ✓ contrainte ${CONTRAINTE} remise"
        psql_ -c "ALTER TABLE ${TABLE} FORCE ROW LEVEL SECURITY;" >/dev/null
        echo "  ✓ FORCE ROW LEVEL SECURITY remis"
    else
        # Déjà présente : c'est le cas normal quand le script se termine sans être interrompu
        # après l'avoir remise, ou quand il échoue avant de l'avoir retirée.
        existe=$(psql_ -c "SELECT COUNT(*) FROM pg_constraint WHERE conname = '${CONTRAINTE}';")
        if [[ "$existe" == "1" ]]; then
            echo "  ✓ contrainte ${CONTRAINTE} déjà en place"
            psql_ -c "ALTER TABLE ${TABLE} FORCE ROW LEVEL SECURITY;" >/dev/null
        else
            echo "  ✗✗✗ LA CONTRAINTE N'A PAS PU ÊTRE REMISE ✗✗✗" >&2
            echo "      La base de développement est SANS sa garantie centrale." >&2
            echo "      Remède : ALTER TABLE ${TABLE} ADD CONSTRAINT ${CONTRAINTE} ${DEFINITION};" >&2
            echo "      ou recharger la base : bash scripts/dev/preparer-base.sh" >&2
            exit 1
        fi
    fi
}
trap remettre EXIT

echo "── P-09 négatif · 1/5 — la porte est-elle verte AVANT ? ──────────────────────"
if ! (cd backend && cargo test --test hebergement_disponibilite >/dev/null 2>&1); then
    echo "  ✗ P-09 est déjà rouge : le test négatif ne prouverait rien." >&2
    echo "    Corriger d'abord, puis relancer." >&2
    exit 1
fi
echo "  ✓ P-09 verte"

echo
echo "── 2/5 — retrait de la contrainte d'exclusion ────────────────────────────────"
psql_ -c "ALTER TABLE ${TABLE} DROP CONSTRAINT ${CONTRAINTE};" >/dev/null
restantes=$(psql_ -c "SELECT COUNT(*) FROM pg_constraint WHERE conname = '${CONTRAINTE}';")
if [[ "$restantes" != "0" ]]; then
    echo "  ✗ la contrainte est toujours là : le retrait a échoué en silence." >&2
    exit 1
fi
echo "  ✓ ${CONTRAINTE} retirée — la double attribution est désormais possible"

echo
echo "── 3/5 — la porte échoue-t-elle, et sur les BONNES assertions ? ──────────────"
sortie=$(cd backend && cargo test --test hebergement_disponibilite 2>&1 || true)

if grep -q "test result: ok" <<<"$sortie"; then
    echo "  ✗✗✗ P-09 EST RESTÉE VERTE SANS LA CONTRAINTE ✗✗✗" >&2
    echo >&2
    echo "  C'est le pire des résultats : la porte ne vérifie pas ce qu'elle annonce." >&2
    echo "  Elle passerait au vert sur une base où deux clients peuvent recevoir la même" >&2
    echo "  chambre — exactement le défaut que le cycle 001 a documenté sur P-08." >&2
    exit 1
fi

echo "  ✓ P-09 est rouge"

# **Elle doit échouer sur les bonnes assertions**, pas sur n'importe laquelle. Une porte qui
# tombe pour la mauvaise raison est indistinguable d'une porte qui fonctionne.
attendus=(
    "p09_assertion_2_une_contrainte_d_exclusion_gist_protege_la_periode"
    "deux_attributions_concurrentes_une_seule_reussit"
    "sc002_l_attribution_echoue_meme_en_contournant_le_service"
)
manquants=()
for test in "${attendus[@]}"; do
    if ! grep -qE "test ${test} \.\.\. FAILED" <<<"$sortie"; then
        manquants+=("$test")
    fi
done

if [[ ${#manquants[@]} -gt 0 ]]; then
    echo "  ✗ P-09 est rouge, mais PAS sur les assertions attendues." >&2
    echo "    N'ont pas échoué : ${manquants[*]}" >&2
    echo >&2
    echo "    Une porte qui tombe pour la mauvaise raison est indistinguable d'une porte qui" >&2
    echo "    fonctionne. Sans la contrainte d'exclusion, ces trois-là DOIVENT échouer :" >&2
    echo "      · l'assertion 2 ne trouve plus la contrainte au catalogue ;" >&2
    echo "      · le test de concurrence voit les deux attributions réussir ;" >&2
    echo "      · l'écriture directe par le repository n'est plus refusée." >&2
    exit 1
fi

echo "  ✓ les trois assertions attendues ont échoué :"
for test in "${attendus[@]}"; do
    echo "      · ${test}"
done

# `remettre` s'exécute par le `trap`.
trap - EXIT
remettre

echo
echo "── 5/5 — la porte est-elle de nouveau verte ? ────────────────────────────────"
if ! (cd backend && cargo test --test hebergement_disponibilite >/dev/null 2>&1); then
    echo "  ✗ P-09 est restée rouge après remise en état." >&2
    echo "    Vérifier la contrainte à la main avant de continuer." >&2
    exit 1
fi
echo "  ✓ P-09 verte"

echo
echo "P-09 négatif ✓ — la porte sait échouer, et sur les bonnes assertions."
